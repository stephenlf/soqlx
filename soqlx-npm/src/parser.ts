interface SoqlxQuery {
    toString(): string
}

interface ParsedQuery {

}

export function parse(soqlxQuery: SoqlxQuery): ParsedQuery {
    let query = '';

    const lines = soqlxQuery.toString().split('\n');
    for (const line of lines) {

    }

    throw new Error('Method not yet implemented.');
}