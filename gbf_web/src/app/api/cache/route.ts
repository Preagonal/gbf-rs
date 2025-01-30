import { cacheable } from '@/lib/cache';

export async function POST() {
    await cacheable.clear();
    return new Response(JSON.stringify({}), { status: 200 });
}