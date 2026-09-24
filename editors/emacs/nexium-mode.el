;;; nexium-mode.el --- Major mode for the Nexium language  -*- lexical-binding: t; -*-

;; Author: Londopy
;; URL: https://github.com/Londopy/nexium
;; Version: 1.2.1
;; Package-Requires: ((emacs "27.1"))
;; Keywords: languages
;; SPDX-License-Identifier: MIT

;;; Commentary:

;; Highlighting, comments, indentation and `nexium-run' / `nexium-test' /
;; `nexium-check' / `nexium-fmt' for `.nx' files, and the `nx lsp' language
;; server registered with Eglot (and, when it is loaded, lsp-mode).
;; Install: see README.md in this directory.

;;; Code:

(require 'rx)

(defgroup nexium nil
  "The Nexium language."
  :group 'languages
  :prefix "nexium-")

(defcustom nexium-nx-command "nx"
  "The Nexium compiler."
  :type 'string
  :group 'nexium)

(defcustom nexium-indent-offset 4
  "Spaces per indentation level, what `nx fmt' writes."
  :type 'integer
  :group 'nexium)

;; ---------------------------------------------------------------- syntax table

(defvar nexium-mode-syntax-table
  (let ((table (make-syntax-table)))
    ;; // comments to end of line
    (modify-syntax-entry ?/ ". 12" table)
    (modify-syntax-entry ?\n ">" table)
    ;; strings and chars
    (modify-syntax-entry ?\" "\"" table)
    (modify-syntax-entry ?' "\"" table)
    (modify-syntax-entry ?\\ "\\" table)
    ;; identifiers
    (modify-syntax-entry ?_ "_" table)
    ;; brackets
    (modify-syntax-entry ?\( "()" table)
    (modify-syntax-entry ?\) ")(" table)
    (modify-syntax-entry ?\[ "(]" table)
    (modify-syntax-entry ?\] ")[" table)
    (modify-syntax-entry ?{ "(}" table)
    (modify-syntax-entry ?} "){" table)
    ;; punctuation
    (dolist (c '(?+ ?- ?* ?% ?& ?| ?^ ?~ ?< ?> ?= ?! ?? ?@ ?. ?: ?,))
      (modify-syntax-entry c "." table))
    table)
  "Syntax table for `nexium-mode'.")

;; ---------------------------------------------------------------- font lock

(defconst nexium--keywords
  '("fn" "let" "var" "const" "struct" "enum" "record" "ref" "class" "trait" "impl"
    "pub" "import" "comptime" "unsafe" "type" "distinct" "where" "into"
    "artifact" "test" "bench" "export" "as" "in" "dyn" "weak" "extern" "derive" "layout" "own"
    "if" "else" "match" "for" "while" "step" "parallel"
    "break" "continue" "return" "defer" "errdefer" "using" "unreachable"
    "try" "catch" "orelse" "and" "or"))

(defconst nexium--constants '("true" "false" "null" "undefined"))

(defconst nexium--primitives
  '("i8" "i16" "i32" "i64" "i128" "u8" "u16" "u32" "u64" "u128" "isize" "usize"
    "f32" "f64" "bool" "char" "void" "never" "Self" "List" "String" "Map"))

(defconst nexium--effects
  '("allocates" "refcounts" "blocks" "shared_mutable" "nondeterministic" "panics" "ffi" "unbounded_stack"))

(defconst nexium--builtins
  '("println" "print" "eprintln" "format" "expect" "expect_eq" "panic"))

(defconst nexium--namespaces
  '("math" "io" "os" "process" "time" "random" "mem" "slice" "utf8" "ascii" "fmt" "context" "alloc"))

(defconst nexium--binary-modifiers
  '("big" "little" "native" "signed" "unsigned" "float" "utf8" "bytes"))

(defconst nexium-font-lock-keywords
  `(;; doc comments (the syntax table makes them comments; this recolors them)
    (,(rx bol (* space) (group (or "///" "//!") (* nonl))) 1 font-lock-doc-face t)
    ;; binary pattern modifiers: 16/big-unsigned
    (,(rx (group "/" (regexp (regexp-opt nexium--binary-modifiers))
                 (* "-" (regexp (regexp-opt nexium--binary-modifiers))))
          word-boundary)
     1 font-lock-keyword-face)
    ;; effects: !allocates
    (,(rx "!" (regexp (regexp-opt nexium--effects)) word-boundary) 0 font-lock-preprocessor-face)
    ;; intrinsics: @typeName(...)
    (,(rx "@" (or "typeName" "sizeOf" "truncate" "errorName" "embedFile" "weak" "refCount" "cImport" "cstr") word-boundary)
     0 font-lock-preprocessor-face)
    ;; error names
    (,(rx word-boundary (group "error") "." (group (+ (any "A-Za-z0-9_"))))
     (1 font-lock-keyword-face) (2 font-lock-constant-face))
    ;; declarations
    (,(rx word-boundary "fn" (+ space) (group (+ (any "A-Za-z0-9_")))) 1 font-lock-function-name-face)
    (,(rx word-boundary (or "struct" "enum" "record" "class" "trait" "type") (+ space) (group (+ (any "A-Za-z0-9_"))))
     1 font-lock-type-face)
    (,(rx word-boundary (or "let" "var" "const") (+ space) (group (+ (any "a-z0-9_")))) 1 font-lock-variable-name-face)
    ;; labels
    (,(rx bol (* space) (group (+ (any "a-z0-9_")) ":") (* space) (or "{" "for" "while")) 1 font-lock-constant-face)
    (,(rx ":" (group (+ (any "a-z0-9_"))) word-boundary) 1 font-lock-constant-face)
    ;; keywords, constants, types
    (,(regexp-opt nexium--keywords 'symbols) . font-lock-keyword-face)
    (,(regexp-opt nexium--constants 'symbols) . font-lock-constant-face)
    (,(regexp-opt nexium--primitives 'symbols) . font-lock-type-face)
    (,(rx word-boundary (group (regexp (regexp-opt nexium--namespaces))) "." (any "a-zA-Z_")) 1 font-lock-builtin-face)
    (,(rx word-boundary (group (any "A-Z") (* (any "A-Za-z0-9_"))) word-boundary) 1 font-lock-type-face)
    ;; variants: .Circle
    (,(rx (not (any "A-Za-z0-9_)]")) "." (group (any "A-Z") (* (any "A-Za-z0-9_")))) 1 font-lock-constant-face)
    ;; calls
    (,(rx word-boundary (group (regexp (regexp-opt nexium--builtins))) "(") 1 font-lock-builtin-face)
    (,(rx word-boundary (group (any "a-z_") (* (any "A-Za-z0-9_"))) "(") 1 font-lock-function-name-face)
    ;; numbers
    (,(rx word-boundary (or (seq "0" (any "xX") (+ (any "0-9A-Fa-f_")))
                            (seq "0" (any "oO") (+ (any "0-7_")))
                            (seq "0" (any "bB") (+ (any "01_")))
                            (seq (any "0-9") (* (any "0-9_")) (? "." (any "0-9") (* (any "0-9_")))
                                 (? (any "eE") (? (any "+-")) (+ (any "0-9_")))))
          word-boundary)
     0 font-lock-constant-face)
    ;; placeholders in strings: {name}
    (nexium--match-placeholder 0 font-lock-variable-name-face t))
  "Font lock rules for `nexium-mode'.")

(defun nexium--match-placeholder (limit)
  "Find the next `{name}' inside a string before LIMIT, for font lock."
  (let (found)
    (while (and (not found)
                (re-search-forward (rx "{" (* (not (any "{}" "\"" ?\n))) "}") limit t))
      (when (save-excursion (nth 3 (syntax-ppss (match-beginning 0))))
        (setq found t)))
    found))

(defun nexium--syntax-propertize (start end)
  "Mark raw strings r\"...\" and bytes b\"...\" between START and END."
  (goto-char start)
  (while (re-search-forward (rx (or "r" "b") "\"") end t)
    (let ((open (match-beginning 0)))
      ;; `syntax-ppss' moves point; the search must resume after the match
      (unless (or (save-excursion (nth 8 (syntax-ppss open)))
                  (memq (char-syntax (or (char-before open) ?\s)) '(?w ?_)))
        ;; the prefix letter opens the string, the quote after it is plain
        (put-text-property open (1+ open) 'syntax-table (string-to-syntax "|"))
        (put-text-property (1+ open) (+ 2 open) 'syntax-table (string-to-syntax "."))))))

;; ---------------------------------------------------------------- indentation

(defun nexium--line-opens ()
  "Net brackets opened by the current line, ignoring comments and strings."
  (save-excursion
    (beginning-of-line)
    (let ((end (line-end-position)) (depth 0))
      (while (< (point) end)
        (let ((state (syntax-ppss)))
          (cond ((or (nth 3 state) (nth 4 state)) nil)
                ((memq (char-after) '(?\( ?\[ ?{)) (setq depth (1+ depth)))
                ((memq (char-after) '(?\) ?\] ?})) (setq depth (max 0 (1- depth))))))
        (forward-char 1))
      depth)))

(defun nexium-indent-line ()
  "Indent the current line.
A bracket opened on the previous line indents by `nexium-indent-offset';
a closing bracket at the start of this line outdents."
  (interactive)
  (let* ((closes (save-excursion
                   (back-to-indentation)
                   (memq (char-after) '(?\) ?\] ?}))))
         (target
          (save-excursion
            (beginning-of-line)
            (if (bobp)
                0
              (forward-line -1)
              (while (and (not (bobp)) (looking-at-p (rx bol (* space) eol)))
                (forward-line -1))
              (+ (current-indentation)
                 (* nexium-indent-offset (nexium--line-opens))
                 (if closes (- nexium-indent-offset) 0))))))
    (setq target (max 0 target))
    (if (<= (current-column) (current-indentation))
        (indent-line-to target)
      (save-excursion (indent-line-to target)))))

;; ---------------------------------------------------------------- commands

(defun nexium--run (subcommand)
  "Run `nx SUBCOMMAND' on the current file in a compilation buffer."
  (let ((file (or (buffer-file-name) (user-error "Save the buffer first"))))
    (save-buffer)
    (compile (format "%s %s %s" nexium-nx-command subcommand (shell-quote-argument file)))))

(defun nexium-run ()
  "Compile and run the current file."
  (interactive)
  (nexium--run "run"))

(defun nexium-test ()
  "Run the tests in the current file."
  (interactive)
  (nexium--run "test"))

(defun nexium-check ()
  "Check the current file; diagnostics go to the compilation buffer."
  (interactive)
  (nexium--run "check"))

(defun nexium-fmt ()
  "Format the current file with `nx fmt' and reload it."
  (interactive)
  (let ((file (or (buffer-file-name) (user-error "Save the buffer first"))))
    (save-buffer)
    (call-process nexium-nx-command nil nil nil "fmt" file)
    (revert-buffer t t t)))

(defvar nexium-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-c C-r") #'nexium-run)
    (define-key map (kbd "C-c C-t") #'nexium-test)
    (define-key map (kbd "C-c C-c") #'nexium-check)
    (define-key map (kbd "C-c C-f") #'nexium-fmt)
    map)
  "Keymap for `nexium-mode'.")

;; the variables and functions of libraries that may not be loaded
(defvar compilation-error-regexp-alist)
(defvar compilation-error-regexp-alist-alist)
(defvar eglot-server-programs)
(defvar lsp-language-id-configuration)
(declare-function lsp-register-client "lsp-mode")
(declare-function make-lsp-client "lsp-mode")
(declare-function lsp-stdio-connection "lsp-mode")
(declare-function lsp-activate-on "lsp-mode")

;; `nx check' diagnostics in compilation buffers:
;;   error: message
;;     --> file.nx:12:5
(with-eval-after-load 'compile
  (add-to-list 'compilation-error-regexp-alist-alist
               '(nexium "^[ \t]*--> \\([^:\n]+\\):\\([0-9]+\\):\\([0-9]+\\)" 1 2 3))
  (add-to-list 'compilation-error-regexp-alist 'nexium))

;; ---------------------------------------------------------------- the mode

;;;###autoload
(define-derived-mode nexium-mode prog-mode "Nexium"
  "Major mode for Nexium source."
  :syntax-table nexium-mode-syntax-table
  (setq-local font-lock-defaults '(nexium-font-lock-keywords))
  (setq-local syntax-propertize-function #'nexium--syntax-propertize)
  (setq-local comment-start "// ")
  (setq-local comment-start-skip "//+!?[ \t]*")
  (setq-local comment-end "")
  (setq-local indent-line-function #'nexium-indent-line)
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width nexium-indent-offset)
  (setq-local electric-indent-chars (append "{}()[]" electric-indent-chars)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.nx\\'" . nexium-mode))

;; ---------------------------------------------------------------- language server

;;;###autoload
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(nexium-mode . ("nx" "lsp"))))

;;;###autoload
(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration '(nexium-mode . "nexium"))
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection '("nx" "lsp"))
                    :activation-fn (lsp-activate-on "nexium")
                    :server-id 'nexium)))

(provide 'nexium-mode)
;;; nexium-mode.el ends here
