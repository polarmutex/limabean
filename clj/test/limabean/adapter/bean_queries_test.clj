(ns limabean.adapter.bean-queries-test
  (:require [clojure.test :refer [deftest is]]
            [limabean.adapter.bean-queries :as sut]))

(deftest errors-nil-when-no-errors
  (is (nil? (sut/errors {})))
  (is (nil? (sut/errors {:plugins [{:name "good-plugin"}]}))))

(deftest errors-parser
  (let [parse-error {:message "unexpected token" :span [1 5]}]
    (is (= {:parser [parse-error]}
           (sut/errors {:error {:parser [parse-error]}})))))

(deftest errors-plugin-load
  (is (= {:plugins [{:name "bad-plugin" :message "plugin not found"}]}
         (sut/errors {:plugins [{:name "bad-plugin" :err {:message "plugin not found"}}
                                {:name "good-plugin"}]}))))

(deftest errors-booking
  (let [booking-err {:spanned-reports [{:message "invalid transaction"}]}]
    (is (= {:booking booking-err}
           (sut/errors {:error {:booking booking-err}})))))

(deftest errors-plugin-execution
  (let [exception {:cause "NullPointerException" :via []}]
    (is (= {:raw-plugin {:exception exception}}
           (sut/errors {:error {:raw-plugin {:exception exception}}})))
    (is (= {:booked-plugin {:exception exception}}
           (sut/errors {:error {:booked-plugin {:exception exception}}})))))
