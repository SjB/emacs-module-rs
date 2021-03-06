(require 't)

(ert-deftest inc ()
  (should (= (t/inc 3) 4))
  (should (equal (documentation 't/inc) "1+"))

  (should-error (t/inc "3") :type 'wrong-type-argument)
  (should-error (t/inc nil) :type 'wrong-type-argument)

  (should-error (t/inc) :type 'wrong-number-of-arguments)
  (should-error (t/inc 1 2) :type 'wrong-number-of-arguments))

(ert-deftest propagate-errors ()
  (should-error (t/calling-error) :type 'arith-error))

(ert-deftest passthrough ()
  (let ((x "x"))
    (should (eq (t/identity x) x))))

(ert-deftest create-function ()
  (let ((dec (t/make-dec)))
    (should (= (funcall dec 9) 8))
    (should (equal (documentation dec) "decrement"))
    (should-error (funcall dec) :type 'wrong-number-of-arguments))
  (let* ((fns (t/make-inc-and-plus))
         (inc (car fns))
         (plus (cdr fns)))
    (should (= (funcall inc -2) -1))
    (should (equal (documentation inc) "increment"))
    (should-error (funcall inc) :type 'wrong-number-of-arguments)

    (should (= (funcall plus 3 5) 8))
    (should (equal (documentation plus) ""))
    (should-error (funcall plus "s" 5) :type 'wrong-type-argument)))

(ert-deftest from-emacs-string ()
  (should (equal (t/to-uppercase "abc") "ABC")))

(ert-deftest user-ptr ()

  (let* ((v1 (t/vector:make 5 6))
         (v2 (t/vector:make 1 3)))
    (should (string-prefix-p "#<user-ptr" (format "%s" v1)))
    (should (equal (t/vector:to-list v1) '(5 6)))
    (should (equal (t/vector:to-list
                    (t/vector:add v1 v2))
                   '(6 9)))
    ;; Emacs doesn't support custom equality...
    (should (not (equal (t/vector:add v1 v2)
                        (t/vector:add v1 v2))))
    ;; ... but should consider an object equal to itself.
    (should (let* ((v3 (t/vector:add v1 v2))
                   (v4 (t/identity v3)))
              (equal v3 v4))))

  ;; Mutation.
  (let ((v (t/vector:make 5 6)))
    (t/vector:scale-mutably 3 v)
    (should (equal (t/vector:to-list v) '(15 18)))
    (t/vector:swap-components v)
    (should (equal (t/vector:to-list v) '(18 15))))

  ;; ;; This is to trigger the finalizer. TODO: Somehow validate they actually runs.
  ;; (dotimes (i 100)
  ;;   (t/vector:make 1 2))
  ;; (garbage-collect)

  (let ((s (t/wrap-string "abc")))
    (should (string-prefix-p "#<user-ptr" (format "%s" s)))
    ;; TODO: :type
    (should-error (t/vector:to-list s))))

(ert-deftest ref-cell ()
  (let ((x (t/refcell:make 5))
        (v (t/vector:make 1 2)))
    ;; TODO: :type
    (should-error (t/refcell:mutate-twice v))
    (should-error (t/refcell:mutate-twice x))))

(ert-deftest simplified-fns ()
  (let ((x 5)
        (y 3))
    (should (equal (t/sum-and-diff x y)
                   (list (+ x y) (- x y))))))

(ert-deftest flexible-return-type ()
  (let ((x 3)
        (y 4))
    (should (equal (t/sum x y)
                   (+ x y)))))

(ert-deftest hash-map ()
  (let ((m (t/hash-map:make)))
    (should (equal (t/hash-map:get m "a") nil))

    (should (equal (t/hash-map:set m "a" "1") nil))
    (should (equal (t/hash-map:get m "a") "1"))

    (should (equal (t/hash-map:set m "a" "2") "1"))
    (should (equal (t/hash-map:get m "a") "2"))))
