const { pathsToModuleNameMapper } = require('ts-jest');
const requireJSON5 = require('require-json5');
const tsconfig = requireJSON5('./tsconfig.json');

/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  moduleNameMapper: {
    "@src/(.*)": ["<rootDir>/src/$1"]
  }
};