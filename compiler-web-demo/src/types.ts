export interface CompileResult {
    tokens: Spanned<any>[];
    ast: Program;
    ir: string;
} 

export interface Position {
    abs: number;
    row: number;
    column: number;
}

export interface Span {
    start: Position;
    end: Position;
}

export interface Spanned<T> {
    value: T;
    span: Span;
}

export interface Program {
    data_types: [string, any][],
    custom_types: [string, Spanned<any>][],
    functions: [string, Spanned<any>][],
    require_main: boolean;
}