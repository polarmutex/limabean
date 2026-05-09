use beancount_parser_lima as parser;

pub(crate) fn book<'a, 'r, 'b>(
    directives: &'r [Directive<'a>],
    options: &parser::Options<'a>,
) -> Result<BookingSuccess<'b>, BookingFailure>
where
    'a: 'b,
    'r: 'b,
{
    let default_booking = limabean_booking::Booking::default();
    let default_booking_option = if let Some(booking_method) = options.booking_method() {
        let booking = Into::<limabean_booking::Booking>::into(*booking_method.item());
        if limabean_booking::is_supported_method(booking) {
            booking
        } else {
            // TODO warning
            // warnings.push(booking_method.warning(format!(
            //     "Unsupported booking method, falling back to {default_booking}"
            // )));
            default_booking
        }
    } else {
        default_booking
    };

    let infer_tolerance_from_cost = options
        .infer_tolerance_from_cost()
        .map(|x| *x.item())
        .unwrap_or(false);

    let tolerance = options.into();

    Accumulator::new(default_booking_option, &tolerance, infer_tolerance_from_cost)
        .collect(directives)
}

mod accumulator;
mod tests;
mod types;

pub(crate) use crate::api::booking::accumulator::{Accumulator, BookingFailure, BookingSuccess};
use crate::api::types::raw::Directive;

#[cfg(test)]
mod tests {
    use beancount_parser_lima::{BeancountParser, BeancountSources, ParseSuccess};

    use crate::api::types::raw;

    fn book_str_ok(source: &str) -> bool {
        let sources = BeancountSources::from(source);
        let parser = BeancountParser::new(&sources);
        let ParseSuccess {
            directives,
            options,
            ..
        } = parser.parse().expect("parse should succeed");
        let raw_directives: Vec<raw::Directive<'_>> =
            directives.iter().map(Into::into).collect();
        super::book(&raw_directives, &options).is_ok()
    }

    #[test]
    fn infer_tolerance_from_cost_allows_small_residual() {
        // 86.216 × 91.6303 = 7899.9979448, paid 7900: residual 0.002 USD.
        // beancount tolerance with infer_tolerance_from_cost:
        //   0.5 × 10^(-4) × 86.216 = 0.0043108 USD  → should pass
        assert!(
            book_str_ok(
                r#"option "infer_tolerance_from_cost" "TRUE"
2015-01-01 open Assets:IRA:TICKER
2015-01-01 open Assets:IRA:USD
2015-03-03 * "Buy"
  Assets:IRA:TICKER  86.216 TICKER {91.6303 USD}
  Assets:IRA:USD  -7900 USD
"#
            ),
            "expected booking to succeed with infer_tolerance_from_cost=TRUE"
        );
    }

    #[test]
    fn without_infer_tolerance_from_cost_residual_is_error() {
        // Same transaction but without the option: residual 0.002 > tolerance 0.0005 → error
        assert!(
            !book_str_ok(
                r#"2015-01-01 open Assets:IRA:TICKER
2015-01-01 open Assets:IRA:USD
2015-03-03 * "Buy"
  Assets:IRA:TICKER  86.216 TICKER {91.6303 USD}
  Assets:IRA:USD  -7900 USD
"#
            ),
            "expected booking to fail without infer_tolerance_from_cost"
        );
    }
}
