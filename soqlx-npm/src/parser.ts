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
            throw new SoqlxParserError('')
        }
        this.params.set(key, value);
    }
}

class SoqlxParserError extends Error {}

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

function consumeInlineParams(lines: string[]): SoqlxOptions {
    const options = new SoqlxOptionsFactory();

    let line = lines.pop()!.trim();

    while (line.substring(3) === '///') {
        const tokens = line.split(/\s/);
        let option = null;
        if (tokens[1].startsWith('@')) {
            option = tokens[1].substring(1, tokens[1].length);
        }
        if (option === 'param') {
            options.params?.set(tokens[2], tokens[3])
        }
    }
}

