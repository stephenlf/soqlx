import parse, { parseParamFileArg, SoqlxParserError } from 'src/parser';
import { describe, expect, test } from '@jest/globals'

describe('parseParamFileArg module', () => {
    test('good input: with comment', () => {
        const arg = "'./src/index.ts' is a valid file path"
        const filePath = parseParamFileArg(arg);
        expect(filePath).toBe('./src/index.ts');
    });

    test('good input: with spaces and comment', () => {
        const arg = "'./test-data/This is a file.txt' This file should work";
        const filePath = parseParamFileArg(arg);
        expect(filePath).toBe('./test-data/This is a file.txt');
    })

    test('good input: double quotes', () => {
        const arg = "\"./test-data/This is a file.txt\"";
        const filePath = parseParamFileArg(arg);
        expect(filePath).toBe('./test-data/This is a file.txt');  
    });

    test('bad data: empty string', () => {
        const arg = '';
        expect(() => {parseParamFileArg(arg)})
            .toThrow(SoqlxParserError);
    });

    test('bad output: file doesn\'t exist', () => {
        const arg = "./invalid.invalid";
        expect(() => {parseParamFileArg(arg)})
            .toThrow(SoqlxParserError);
    });

    test('bad output: path is dir', () => {
        const arg = "./src";
        expect(() => {parseParamFileArg(arg)})
            .toThrow(SoqlxParserError);
    });
});