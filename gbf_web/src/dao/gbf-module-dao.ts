export class GbfModuleDao {
    /**
     * GBF version used to decompile the module.
     */
    public gbfVersion: string;

    /**
     * The module ID of the module (SHA256).
     */
    public moduleId: string;

    /**
     * The file name of the module.
     */
    public fileName: string;

    /**
     * The time it took to load the module (in millis).
     */
    public moduleLoadTimeMillis: number;

    /**
     * If the module's decompilation was successful.
     */
    public decompileSuccess: boolean;

    constructor(opts: {
        gbfVersion: string;
        moduleId: string;
        fileName: string;
        moduleLoadTimeMillis: number;
        decompileSuccess: boolean;
    }) {
        this.gbfVersion = opts.gbfVersion;
        this.moduleId = opts.moduleId;
        this.fileName = opts.fileName;
        this.moduleLoadTimeMillis = opts.moduleLoadTimeMillis;
        this.decompileSuccess = opts.decompileSuccess;
    }

    /**
     * Partition Key name.
     */
    public static pkKey(): string {
        return 'gbf_version';
    }

    /**
     * PK value. 
     */
    public pkVal(): string {
        return this.gbfVersion;
    }

    /**
     * Optionally, a static or dynamic Sort Key name. 
     * If you're storing (version, moduleId) as PK/SK, do something like:
     */
    public static skKey(): string {
        return 'module_id';
    }

    public skVal(): string {
        return this.moduleId;
    }

    public static toPlainObject(instance: GbfModuleDao) {
        return {
            gbfVersion: instance.gbfVersion,
            moduleId: instance.moduleId,
            fileName: instance.fileName,
            moduleLoadTimeMillis: instance.moduleLoadTimeMillis,
            decompileSuccess: instance.decompileSuccess,
        };
    }

}
