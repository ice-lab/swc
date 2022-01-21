module.exports = {
  // 'testRunner': 'jest-circus/runner',
  coverageDirectory: './coverage/',
  testEnvironment: 'node',
  collectCoverage: true,
  collectCoverageFrom: ['lib/*.{js,jsx}'],
  coveragePathIgnorePatterns: [
    '<rootDir>/node_modules/',
  ],
  transform: {
    '^.+\\.(j|t)sx?$': 'ts-jest',
  },
  roots: [
    '<rootDir>/tests',
  ],
  testPathIgnorePatterns: [
    '/node_modules/',
    '/lib/',
  ],
  bail: 1,
  // copy from jest config
  testMatch: ['**/?(*.)+(spec|test).[jt]s?(x)'],
};
