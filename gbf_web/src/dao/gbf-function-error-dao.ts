export interface GbfSimplifiedBacktraceFrame {
    function: string;
    file: string;
    line: number;
}

export interface GbfSimplifiedBacktrace {
    frames: GbfSimplifiedBacktraceFrame[];
}

export class GbfFunctionErrorDao {
    /**
     * GBF version
     */
    public gbfVersion: string;

    /**
     * Module ID
     */
    public moduleId: string;

    /**
     * The function address that encountered the error.
     */
    public functionAddress: number; // or string

    /**
     * The type of error (e.g. structure analysis, parse error, etc.)
     */
    public errorType: string;

    /**
     * A human-readable message or summary.
     */
    public message: string;

    /**
     * A structured backtrace
     */
    public backtrace: GbfSimplifiedBacktrace;

    /**
     * A more detailed error context
     */
    public errorContext: string;

    constructor(opts: {
        gbfVersion: string;
        moduleId: string;
        functionAddress: number;
        errorType: string;
        message: string;
        backtrace: GbfSimplifiedBacktrace;
        errorContext: string;
    }) {
        this.gbfVersion = opts.gbfVersion;
        this.moduleId = opts.moduleId;
        this.functionAddress = opts.functionAddress;
        this.errorType = opts.errorType;
        this.message = opts.message;
        this.backtrace = opts.backtrace;
        this.errorContext = opts.errorContext;
    }

    public static pkKey(): string {
        return 'gbf_version#module_id';
    }

    public pkVal(): string {
        return `${this.gbfVersion}#${this.moduleId}`;
    }

    public static skKey(): string {
        return 'function_address';
    }

    public skVal(): string {
        return String(this.functionAddress);
    }

    public static toPlainObject(instance: GbfFunctionErrorDao) {
        return {
            gbfVersion: instance.gbfVersion,
            moduleId: instance.moduleId,
            functionAddress: instance.functionAddress,
            errorType: instance.errorType,
            message: instance.message,
            backtrace: instance.backtrace,
            errorContext: instance.errorContext,
        };
    }
}
