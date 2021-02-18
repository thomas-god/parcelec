module.exports = {
  parser: '@typescript-eslint/parser',
  parserOptions: {
    project: './tsconfig.json',
    ecmaVersion: 2018, // Allows for the parsing of modern ECMAScript features
    sourceType: 'module', // Allows for the use of imports
  },
  plugins: ['@typescript-eslint', 'import'],
  extends: [
    'eslint:recommended',
    'prettier',
    'plugin:prettier/recommended',
    'prettier/@typescript-eslint',
  ],
  env: {
    // avoid getting lint errors when using Promise.
    es6: true,
    mocha: true,
    node: true,
  },
  rules: {
    'no-extra-semi': 2,
    eqeqeq: [1, 'smart'],
    'no-console': 0,
    // curly braces are mandatory (if, for, while blocks...)
    curly: 2,
    // enforce if / else structure
    'brace-style': ['error', '1tbs', { allowSingleLine: false }],
    // enforce return statement in array methods
    'array-callback-return': 2,
    // avoid else if possible (in if / else statement)
    'no-else-return': 2,
    // enforce type checking when comparing to null
    'no-eq-null': 2,
    'import/exports-last': 2,
    'import/order': ['error'],
  },
  overrides: [
    {
      files: ['**/*.ts'],
      rules: {
        /**
         * Recommended rules. 1.13.0
         * We must specify them manually because extends both js and typescript rules
         * won't work. We can't specify extends in "overrides" object
         */
        '@typescript-eslint/adjacent-overload-signatures': 'error',
        '@typescript-eslint/array-type': 'error',
        '@typescript-eslint/ban-types': 'error',
        camelcase: 'off',
        '@typescript-eslint/camelcase': 'error',
        '@typescript-eslint/class-name-casing': 'error',
        '@typescript-eslint/explicit-function-return-type': 'warn',
        '@typescript-eslint/explicit-member-accessibility': 'error',
        '@typescript-eslint/interface-name-prefix': 'error',
        '@typescript-eslint/member-delimiter-style': 'error',
        // removed in latest eslint versions
        // '@typescript-eslint/no-angle-bracket-type-assertion': 'error',
        'no-array-constructor': 'off',
        '@typescript-eslint/no-array-constructor': 'error',
        '@typescript-eslint/no-empty-interface': 'error',
        '@typescript-eslint/no-explicit-any': 'warn',
        '@typescript-eslint/no-inferrable-types': 'error',
        '@typescript-eslint/no-misused-new': 'error',
        '@typescript-eslint/no-namespace': 'error',
        '@typescript-eslint/no-non-null-assertion': 'error',
        // removed in latest eslint versions
        // '@typescript-eslint/no-object-literal-type-assertion': 'error',
        '@typescript-eslint/no-parameter-properties': 'error',
        // removed in latest eslint versions
        // '@typescript-eslint/no-triple-slash-reference': 'error',
        'no-unused-vars': 'off',
        '@typescript-eslint/no-unused-vars': 'warn',
        'no-use-before-define': 'off',
        '@typescript-eslint/no-use-before-define': 'error',
        '@typescript-eslint/no-var-requires': 'error',
        // removed in latest eslint versions
        // '@typescript-eslint/prefer-interface': 'error',
        '@typescript-eslint/prefer-namespace-keyword': 'error',
        '@typescript-eslint/type-annotation-spacing': 'error',

        /**
         * Custom rules
         */
        // won't work with destructured constructor since we cannot specify private / protected
        '@typescript-eslint/explicit-member-accessibility': 'off',
        // we can't alias an interface if this rule is active
        '@typescript-eslint/no-empty-interface': 'off',
        // Warning instead of error for migration purposes
        '@typescript-eslint/no-var-requires': 1,
        '@typescript-eslint/explicit-function-return-type': [
          'warn',
          {
            // avoid warning .forEach for missing return.
            allowExpressions: true,
          },
        ],
        'object-curly-spacing': ['error', 'always'],
      },
    },
  ],
};