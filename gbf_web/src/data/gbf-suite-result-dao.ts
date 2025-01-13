export interface DecompileTime {
    nanos: number;
    secs: number;
}

export interface DecompilerError {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    obj: any;
}

export interface FunctionId {
    version: string;
    moduleId: string;
    functionName: string | null;
    functionAddress: number;
}

export interface ModuleId {
    version: string;
    moduleId: string;
    fileName: string;
}

export interface FunctionResult {
    functionId: FunctionId;
    decompileTime: DecompileTime;
    svgCfg: SvgRef;
    decompiled: string | DecompilerError;
    decompileSuccess: boolean;
}

export interface SvgRef {
    Text?: string;
    Key?: string;
}

export interface ModuleResult {
    moduleId: ModuleId;
    moduleLoadTime: DecompileTime;
    functions: Array<FunctionResult>;
    decompileSuccess: boolean;
}

export interface GbfSuiteResult {
    version: string;
    totalTime: DecompileTime;
    modules: ModuleResult[];
    decompileSuccess: boolean;
}


/* eslint-disable @typescript-eslint/no-explicit-any */
export function parseGbfSuiteResults(data: any[]): GbfSuiteResult[] {
    if (!Array.isArray(data)) {
        throw new Error("Invalid data format: Expected an array.");
    }

    return data.map((item: any) => {
        if (typeof item !== "object" || item === null) {
            throw new Error("Invalid data format: Expected an object.");
        }

        const current_suite_run = item.module_json;


        const modules: ModuleResult[] = current_suite_run.modules.map((module: any) => {
            const moduleLoadTime: DecompileTime = module.module_load_time;

            const moduleId: ModuleId = {
                version: module.gbf_version,
                moduleId: module.module_id,
                fileName: module.file_name,
            };

            const functions: Array<FunctionResult> =
                module.functions.Ok?.map((func: any) => {
                    let decompiled: string | DecompilerError;
                    if (func.decompile_result.Err) {
                        decompiled = { obj: func.decompile_result.Err } as DecompilerError;
                    } else {
                        decompiled = func.decompile_result.Ok as string;
                    }

                    const functionId: FunctionId = {
                        version: func.gbf_version,
                        moduleId: func.module_id,
                        functionName: func.function_name || null,
                        functionAddress: func.function_address,
                    };

                    const svgCfg: SvgRef = func.svg_cfg.Text
                        ? { Text: func.svg_cfg.Text }
                        : { Key: func.svg_cfg.Key };

                    return {
                        functionId,
                        decompileTime: func.decompile_time,
                        svgCfg,
                        decompiled,
                        decompileSuccess: func.decompile_success,
                    };
                }) || [];

            return {
                moduleLoadTime,
                functions,
                decompileSuccess: module.decompile_success,
                moduleId: moduleId,
            };
        });

        return {
            version: current_suite_run.gbf_version,
            totalTime: current_suite_run.total_time,
            modules,
            decompileSuccess: current_suite_run.decompile_success,
        };
    });
}
