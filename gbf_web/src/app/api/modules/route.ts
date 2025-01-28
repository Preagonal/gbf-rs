import { getAllModules, getModule } from "@/lib/module";
import { NextRequest } from "next/server";

export async function GET(request: NextRequest) {
    const searchParams = request.nextUrl.searchParams;
    const versionId = searchParams.get('version');
    if (!versionId) {
        return new Response(JSON.stringify({}), { status: 400 });
    }
    const moduleId = searchParams.get('module');
    if (!moduleId) {
        const modules = await getAllModules(versionId);
        return new Response(JSON.stringify(modules), { status: 200 });
    }

    const mod = await getModule(versionId, moduleId);
    if (!mod) {
        return new Response(JSON.stringify({}), { status: 404 });
    }
    return new Response(JSON.stringify(mod), { status: 200 });
}