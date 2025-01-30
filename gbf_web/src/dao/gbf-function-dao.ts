export class GbfFunctionDao {
    /**
     * The GBF version used to decompile the function.
     */
    public gbfVersion: string;

    /**
     * The module ID to which this function belongs.
     */
    public moduleId: string;

    /**
     * The function address (unique within the module).
     */
    public functionAddress: number; // or string

    /**
     * The name of the function, if known.
     */
    public functionName?: string | null;

    /**
     * Whether the function was decompiled successfully.
     */
    public decompileSuccess: boolean;

    /**
     * The result of the decompilation attempt (could be an error).
     */
    public decompileResult?: string; // or any if you want to store an error object

    /**
     * How long it took to decompile this function (in millis).
     */
    public totalTimeMillis: number;

    /**
     * The S3 URL of the dot file (renamed from dot_key).
     */
    public dotUrl: string;

    constructor(opts: {
        gbfVersion: string;
        moduleId: string;
        functionAddress: number;
        functionName?: string | null;
        decompileSuccess: boolean;
        decompileResult?: string;
        totalTimeMillis: number;
        dotUrl: string;
    }) {
        this.gbfVersion = opts.gbfVersion;
        this.moduleId = opts.moduleId;
        this.functionAddress = opts.functionAddress;
        this.functionName = opts.functionName;
        this.decompileSuccess = opts.decompileSuccess;
        this.decompileResult = opts.decompileResult;
        this.totalTimeMillis = opts.totalTimeMillis;
        this.dotUrl = opts.dotUrl;
    }

    public static pkKey(): string {
        return 'gbf_version#module_id';
    }

    public pkVal(): string {
        return `${this.gbfVersion}#${this.moduleId}`;
    }

    /**
     * If you want a dynamic SK:
     */
    public static skKey(): string {
        return 'function_address';
    }

    public skVal(): string {
        return String(this.functionAddress);
    }

    public static toPlainObject(instance: GbfFunctionDao) {
        return {
            gbfVersion: instance.gbfVersion,
            moduleId: instance.moduleId,
            functionAddress: instance.functionAddress,
            functionName: instance.functionName,
            decompileSuccess: instance.decompileSuccess,
            decompileResult: instance.decompileResult,
            totalTimeMillis: instance.totalTimeMillis,
            dotUrl: instance.dotUrl,
        };
    }
}
