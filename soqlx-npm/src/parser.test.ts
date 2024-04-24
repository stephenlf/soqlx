import parse, { parseParamFileArg, SoqlxParserError } from './parser';
import { describe, expect, test } from '@jest/globals'

describe('parseParamFileArg module', () => {
    test('good input', () => {
        const arg = "'./src/index.ts' is a valid file path"
        const filePath = parseParamFileArg(arg);
        expect(filePath).toBe('./src/index.ts');
    });

    test('bad output: file doesn\'t exist', () => {
        const arg = "./invalid.invalid";
        expect(() => {parseParamFileArg(arg)})
            .toThrow(SoqlxParserError);
    });
});