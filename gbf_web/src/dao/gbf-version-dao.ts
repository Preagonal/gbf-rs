export class GbfVersionDao {
    /**
     * The version of the GBF used.
     */
    public gbfVersion: string;

    /**
     * The total time it took to run the entire suite (in millis).
     */
    public totalTimeMillis: number;

    /**
     * Timestamp (e.g., "2025-01-28T12:34:56Z" or an epoch).
     */
    public suiteTimestamp: number; // or string for ISO8601

    constructor(opts: {
        gbfVersion: string;
        totalTimeMillis: number;
        suiteTimestamp: number;
    }) {
        this.gbfVersion = opts.gbfVersion;
        this.totalTimeMillis = opts.totalTimeMillis;
        this.suiteTimestamp = opts.suiteTimestamp;
    }

    /**
     * A static key name for the Partition Key in DynamoDB.
     */
    public static pkKey(): string {
        return 'gbf_version';
    }

    /**
     * The actual PK value.
     */
    public pkVal(): string {
        return this.gbfVersion;
    }

    toPlainObject() {
        return {
            gbfVersion: this.gbfVersion,
            totalTimeMillis: this.totalTimeMillis,
            suiteTimestamp: this.suiteTimestamp,
        };
    }
}
