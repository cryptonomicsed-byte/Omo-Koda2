import { NextResponse } from 'next/server'

// Server-side proxy to the Omo-Koda kernel's /v1/think primitive.
// The kernel runs the full perceive→think cognition and returns the response
// text in `tool_output` (ExecutionResponse). Configure the kernel URL via
// OMOKODA_URL (defaults to the local sovereign kernel on :7777).
const KERNEL = process.env.OMOKODA_URL || process.env.NEXT_PUBLIC_OMOKODA_URL || 'http://127.0.0.1:7777'

export async function POST(request: Request) {
  const body = await request.json().catch(() => ({}))
  const message: string = body?.message ?? ''
  const isPrivate: boolean = body?.private ?? false

  if (!message.trim()) {
    return NextResponse.json({ error: 'empty message' }, { status: 400 })
  }

  try {
    const r = await fetch(`${KERNEL}/v1/think`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt: message, private: isPrivate }),
    })

    if (!r.ok) {
      const detail = await r.text().catch(() => '')
      return NextResponse.json(
        { error: `kernel returned ${r.status}`, detail: detail.slice(0, 300) },
        { status: 502 },
      )
    }

    const data = await r.json()
    return NextResponse.json({
      response: data.tool_output ?? '(the agent produced no output — it may have thought privately)',
      receipt_id: data.receipt_id ?? null,
      private_mode: data.private_mode ?? isPrivate,
    })
  } catch (e: unknown) {
    return NextResponse.json(
      { error: 'could not reach the Omo-Koda kernel', detail: String(e).slice(0, 200) },
      { status: 502 },
    )
  }
}
