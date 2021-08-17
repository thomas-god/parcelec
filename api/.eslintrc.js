module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  parserOptions: {
    project: "./tsconfig.json",
    ecmaVersion: 2018,
    sourceType: "module",
  },
  plugins: ["@typescript-eslint"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "prettier",
    "plugin:prettier/recommended",
  ],
  env: {
    es6: true,
    mocha: true,
    node: true,
  },
  rules: {
    "no-extra-semi": 2,
    eqeqeq: [1, "smart"],
    "no-console": 0,
    curly: 2,
    "brace-style": ["error", "1tbs", { allowSingleLine: false }],
    "array-callback-return": 2,
    "no-else-return": 2,
    "no-eq-null": 2,
  },
};
