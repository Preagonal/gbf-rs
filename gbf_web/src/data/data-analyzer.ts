import parseGbfSuiteResult from './fetch-data';
import type { GbfSuiteResult, ModuleResult, FunctionResult } from './gbf-suite-result-dao';

/**
 * Singleton for data storage and access
 */
export class GbfDataAnalyzer {
    private static instance: GbfDataAnalyzer | null = null;

    private dataMap: Map<string, ModuleResult[]>;

    private constructor(suiteResults: GbfSuiteResult[]) {
        this.dataMap = this.buildDataMap(suiteResults);
    }

    private buildDataMap(suiteResults: GbfSuiteResult[]): Map<string, ModuleResult[]> {
        const map = new Map<string, ModuleResult[]>();
        for (const suite of suiteResults) {
            map.set(suite.version, suite.modules);
        }
        return map;
    }

    public static async getInstance(): Promise<GbfDataAnalyzer> {
        if (!this.instance) {
            const suiteResults: GbfSuiteResult[] = await parseGbfSuiteResult().catch((err) => {
                throw new Error(`Failed to initialize GbfDataAnalyzer: ${err.message}`);
            });
            this.instance = new GbfDataAnalyzer(suiteResults);
        }
        return this.instance;
    }

    public getAllVersions(): string[] {
        return Array.from(this.dataMap.keys());
    }

    public getModulesByVersion(version: string): ModuleResult[] {
        return this.dataMap.get(version) || [];
    }

    public getModuleById(version: string, moduleId: string): ModuleResult | undefined {
        const modules = this.getModulesByVersion(version);
        return modules.find((m) => m.moduleId.moduleId === moduleId);
    }

    public getFunctionByAddress(version: string, moduleId: string, functionAddress: number): FunctionResult | undefined {
        const moduleObj = this.getModuleById(version, moduleId);
        return moduleObj?.functions.find((fn) => fn.functionId.functionAddress === functionAddress);
    }
}

/**
 * Stateful builder for queries
 */
export class QueryBuilder {
    private analyzer: GbfDataAnalyzer;
    private _version?: string;
    private _moduleId?: string;
    private _functionAddr?: number;

    constructor(analyzer: GbfDataAnalyzer) {
        this.analyzer = analyzer;
    }

    public version(version: string): this {
        this._version = version;
        return this;
    }

    public module(moduleId: string): this {
        this._moduleId = moduleId;
        return this;
    }

    public function(functionAddr: number): this {
        this._functionAddr = functionAddr;
        return this;
    }

    public execute():
        | string[]              // all versions
        | ModuleResult[]        // modules for a version
        | FunctionResult[]      // all functions of a module
        | FunctionResult        // a specific function
        | undefined {
        if (!this._version) {
            return this.analyzer.getAllVersions();
        }
        if (this._version && !this._moduleId) {
            return this.analyzer.getModulesByVersion(this._version);
        }
        if (this._version && this._moduleId && this._functionAddr === undefined) {
            const moduleObj = this.analyzer.getModuleById(this._version, this._moduleId);
            return moduleObj?.functions;
        }
        if (this._version && this._moduleId && this._functionAddr !== undefined) {
            return this.analyzer.getFunctionByAddress(this._version, this._moduleId, this._functionAddr);
        }
        return undefined;
    }
}

/**
 * Factory to create a query builder
 */
export async function createQueryBuilder(): Promise<QueryBuilder> {
    const analyzer = await GbfDataAnalyzer.getInstance();
    return new QueryBuilder(analyzer);
}
