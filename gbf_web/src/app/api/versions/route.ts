import { getAllVersions } from "@/lib/version";

export async function GET() {
    const versions = await getAllVersions();
    return new Response(JSON.stringify(versions), { status: 200 });
}