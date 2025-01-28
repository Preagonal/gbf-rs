export class GbfGraphvizStructureAnalysisDao {
    /**
     * The version of the GBF used to generate the dot file.
     */
    public gbfVersion: string;

    /**
     * The module ID of the module (SHA256 hash of the module).
     */
    public moduleId: string;

    /**
     * The function address of the function (can be used as a unique identifier).
     */
    public functionAddress: number; // or string, depending on Gs2BytecodeAddress usage

    /**
     * The structure analysis step.
     */
    public structureAnalysisStep: number;

    /**
     * The S3 URL of the dot file.
     */
    public dotUrl: string;

    constructor(opts: {
        gbfVersion: string;
        moduleId: string;
        functionAddress: number;
        structureAnalysisStep: number;
        dotUrl: string;
    }) {
        this.gbfVersion = opts.gbfVersion;
        this.moduleId = opts.moduleId;
        this.functionAddress = opts.functionAddress;
        this.structureAnalysisStep = opts.structureAnalysisStep;
        this.dotUrl = opts.dotUrl;
    }

    /**
     * A static key name for the Partition Key in DynamoDB (for clarity).
     */
    public static pkKey(): string {
        return 'gbf_version#module_id#function_address';
    }

    /**
     * The actual Partition Key value for this record.
     */
    public pkVal(): string {
        return `${this.gbfVersion}#${this.moduleId}#${this.functionAddress}`;
    }

    toPlainObject() {
        return {
            gbfVersion: this.gbfVersion,
            moduleId: this.moduleId,
            functionAddress: this.functionAddress,
            structureAnalysisStep: this.structureAnalysisStep,
            dotUrl: this.dotUrl,
        };
    }
}
