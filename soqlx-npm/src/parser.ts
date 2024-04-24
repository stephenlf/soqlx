import { parse as shellParse } from "shell-quote";
import fs from 'node:fs';

type Params = Map<string, string>

export interface SoqlxOptions {
    paramsFile?: string;
    params?: Params;
    pretty?: boolean;
}

class SoqlxOptionsFactory {
    paramsFile?: string;
    params: Params;
    pretty?: boolean;

    constructor() {
        this.params = new Map();
    }

    asOptions(): SoqlxOptions {
        return {...this};
    }

    addParam(key: string, value: string) {
        if (! key || ! value) {
            throw new SoqlxParserError('param requires two arguments')
        }
        this.params.set(key, value);
    }
}

export class SoqlxParserError extends Error {}

interface SoqlxQuery {
    query: {toString(): string},
    options: SoqlxOptions
}

interface ParsedQuery {
    queryString: string,
}

export function parse(soqlxQuery: SoqlxQuery): ParsedQuery {
    // Iterate over lines, from start to finish.
    const lines = soqlxQuery.toString().split('\n');
    if (lines.length == 0) {
        return {queryString: ''};
    }

    lines.reverse();

    throw new Error('Method not yet implemented.');
}

function consumeInlineCommands(lines: string[]): SoqlxOptions {
    const options = new SoqlxOptionsFactory();

    let line = lines.pop()!.trim();

    while (line.substring(3) === '///') {
        const tokens = line.split(/\s/, 2);

    }
}

function parseInlineParam(line: string | undefined) {
    if (! line || ! line.startsWith('@') ) {
        return;
    }
    const [command, argument] = line.split(/\s/, 2);
    switch (command) {
        case '@param':
            
            break;
        case '@paramFile':
            const filePath = parseParamFileArg(argument);
            
            break;
        case '@pretty':

            break;
        default:
            console.warn(`Unrecognized command: ${command}`);
            break;
    }
    return;
}

function parseParamFileArg(argument: string | undefined): string {
    if (! argument) {
        throw new SoqlxParserError('@paramFile missing argument <PATH>');
    }
    const filePath = shellParse(argument)[0].toString();
    if (! fs.statSync(filePath).isFile() ) {
        throw new SoqlxParserError(`@paramFile PATH argument does not point to a valid file: ${filePath}`);
    }
    return filePath;
} 