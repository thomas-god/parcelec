module.exports = {
    testEnvironment: 'node',
    modulePathIgnorePatterns: ["<rootDir>/dist/"],
    coverageReporters: ['lcov', 'html', 'text-summary', 'text'],
    collectCoverageFrom: [
      'src/**/*.ts',
    ],
    transform: {
      '^.+\\.tsx?$': 'ts-jest',
    },
  };