module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testPathIgnorePatterns: [
    "<rootDir>/lib/",
    "<rootDir>/node_modules/"
  ],
  coverageDirectory: "./coverage/",
  collectCoverageFrom: [
    "src/**/*.ts"
  ]
};