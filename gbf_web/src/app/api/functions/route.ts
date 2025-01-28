import { getAllFunctions, getFunction } from "@/lib/function";
import { NextRequest } from "next/server";

export async function GET(request: NextRequest) {
    const searchParams = request.nextUrl.searchParams;
    const versionId = searchParams.get('version');
    const moduleId = searchParams.get('module');
    const functionIdParam = searchParams.get('function');
    if (!versionId || !moduleId) {
        return new Response(JSON.stringify({}), { status: 400 });
    }

    if (!functionIdParam) {
        const funcs = await getAllFunctions(versionId, moduleId);
        return new Response(JSON.stringify(funcs), { status: 200 });
    }

    // turn functionId into a number and throw a 400 if parseInt Nan
    const functionId = parseInt(functionIdParam);
    if (isNaN(functionId)) {
        return new Response(JSON.stringify({}), { status: 400 });
    }

    const func = await getFunction(versionId, moduleId, functionId);
    if (!func) {
        return new Response(JSON.stringify({}), { status: 404 });
    }
    return new Response(JSON.stringify(func), { status: 200 });
}