'use client'

import { useEffect, useRef, useState, useCallback } from 'react'

interface Star {
  id: string
  title: string
  x: number
  y: number
  z: number
  size: number
  color: string
  constellation: string
  tags: string[]
  content_type: string
  path: string
}

interface Edge {
  id: string
  subject: string
  predicate: string
  object: string
  source: [number, number, number]
  target: [number, number, number]
  weight: number
  path: string
}

interface Nebula {
  id: string
  trace_type: string
  x: number
  y: number
  z: number
  opacity: number
  size: number
  path: string
}

export interface GalaxyData {
  agent_name: string
  agent_id: string
  stars: Star[]
  edges: Edge[]
  nebulae: Nebula[]
  clusters: Record<string, Star[]>
  bounds: { min: [number, number, number]; max: [number, number, number] }
}

interface Props {
  data: GalaxyData
  agentName: string
  filter?: string | null
}

interface Camera {
  x: number
  y: number
  zoom: number
}

const EMPTY_GALAXY: GalaxyData = {
  agent_name: '',
  agent_id: '',
  stars: [],
  edges: [],
  nebulae: [],
  clusters: {},
  bounds: { min: [0, 0, 0], max: [1000, 1000, 500] },
}

export function GalaxyViewer({ data = EMPTY_GALAXY, agentName, filter }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const rafRef = useRef<number>(0)
  const cameraRef = useRef<Camera>({ x: 0, y: 0, zoom: 1 })
  const dragRef = useRef<{ dragging: boolean; lastX: number; lastY: number }>({
    dragging: false,
    lastX: 0,
    lastY: 0,
  })
  const [hovered, setHovered] = useState<Star | null>(null)
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 })
  const [activeFilter, setActiveFilter] = useState<string | null>(filter ?? null)

  const stars = data?.stars ?? []
  const edges = data?.edges ?? []
  const nebulae = data?.nebulae ?? []
  const clusters = data?.clusters ?? {}

  const filteredStars = activeFilter
    ? stars.filter(
        (s) => s.constellation === activeFilter || s.content_type === activeFilter
      )
    : stars

  const draw = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const { width, height } = canvas
    const cam = cameraRef.current

    ctx.clearRect(0, 0, width, height)

    // Background
    const bg = ctx.createRadialGradient(
      width / 2,
      height / 2,
      0,
      width / 2,
      height / 2,
      Math.max(width, height) * 0.8
    )
    bg.addColorStop(0, '#0d0d2b')
    bg.addColorStop(1, '#020210')
    ctx.fillStyle = bg
    ctx.fillRect(0, 0, width, height)

    // Coordinate transform: world → screen
    const toScreen = (wx: number, wy: number) => ({
      x: (wx - cam.x) * cam.zoom + width / 2,
      y: (wy - cam.y) * cam.zoom + height / 2,
    })

    // Draw nebulae (background layer)
    for (const neb of nebulae) {
      const sc = toScreen(neb.x, neb.y)
      const r = neb.size * cam.zoom * 0.5
      if (r < 1) continue
      const grad = ctx.createRadialGradient(sc.x, sc.y, 0, sc.x, sc.y, r)
      grad.addColorStop(0, `rgba(102, 51, 153, ${neb.opacity * 0.6})`)
      grad.addColorStop(0.5, `rgba(80, 20, 120, ${neb.opacity * 0.3})`)
      grad.addColorStop(1, 'rgba(0,0,0,0)')
      ctx.fillStyle = grad
      ctx.beginPath()
      ctx.arc(sc.x, sc.y, r, 0, Math.PI * 2)
      ctx.fill()
    }

    // Draw edges (knowledge links)
    for (const edge of edges) {
      const src = toScreen(edge.source[0], edge.source[1])
      const tgt = toScreen(edge.target[0], edge.target[1])
      ctx.strokeStyle = `rgba(0, 240, 255, ${0.1 + edge.weight * 0.15})`
      ctx.lineWidth = Math.max(0.5, edge.weight * 1.5 * cam.zoom)
      ctx.beginPath()
      ctx.moveTo(src.x, src.y)
      ctx.lineTo(tgt.x, tgt.y)
      ctx.stroke()
    }

    // Draw constellation label lines
    for (const [name, clusterStars] of Object.entries(clusters)) {
      if (clusterStars.length < 2) continue
      const cx = clusterStars.reduce((s, st) => s + st.x, 0) / clusterStars.length
      const cy = clusterStars.reduce((s, st) => s + st.y, 0) / clusterStars.length
      const sc = toScreen(cx, cy)
      ctx.fillStyle = 'rgba(150, 200, 255, 0.4)'
      ctx.font = `${Math.max(9, 11 * cam.zoom)}px monospace`
      ctx.textAlign = 'center'
      ctx.fillText(name.toUpperCase(), sc.x, sc.y)
    }

    // Draw stars
    for (const star of filteredStars) {
      const sc = toScreen(star.x, star.y)
      const r = Math.max(2, star.size * 0.4 * cam.zoom)
      const isHov = hovered?.id === star.id

      // Glow effect
      const glowR = r * (isHov ? 4 : 2.5)
      const glow = ctx.createRadialGradient(sc.x, sc.y, 0, sc.x, sc.y, glowR)
      glow.addColorStop(0, star.color + 'cc')
      glow.addColorStop(0.4, star.color + '44')
      glow.addColorStop(1, 'rgba(0,0,0,0)')
      ctx.fillStyle = glow
      ctx.beginPath()
      ctx.arc(sc.x, sc.y, glowR, 0, Math.PI * 2)
      ctx.fill()

      // Core
      ctx.fillStyle = isHov ? '#ffffff' : star.color
      ctx.shadowBlur = isHov ? 18 : 8
      ctx.shadowColor = star.color
      ctx.beginPath()
      ctx.arc(sc.x, sc.y, r, 0, Math.PI * 2)
      ctx.fill()
      ctx.shadowBlur = 0
    }

    // Tooltip
    if (hovered) {
      const px = mousePos.x
      const py = mousePos.y
      const pad = 10
      const w = 200
      const h = 70
      const tx = Math.min(px + 14, width - w - pad)
      const ty = Math.min(py - 10, height - h - pad)

      ctx.fillStyle = 'rgba(10, 10, 30, 0.92)'
      ctx.strokeStyle = '#00f0ff'
      ctx.lineWidth = 1
      ctx.beginPath()
      ctx.roundRect(tx, ty, w, h, 6)
      ctx.fill()
      ctx.stroke()

      ctx.fillStyle = '#00f0ff'
      ctx.font = 'bold 12px monospace'
      ctx.textAlign = 'left'
      ctx.fillText(hovered.title.slice(0, 26), tx + pad, ty + 22)
      ctx.fillStyle = '#8888aa'
      ctx.font = '10px monospace'
      ctx.fillText(`[${hovered.content_type}] ${hovered.constellation}`, tx + pad, ty + 40)
      ctx.fillText(hovered.tags.slice(0, 3).join(', '), tx + pad, ty + 56)
    }

    rafRef.current = requestAnimationFrame(draw)
  }, [filteredStars, edges, nebulae, clusters, hovered, mousePos])

  // Resize observer
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ro = new ResizeObserver(() => {
      canvas.width = canvas.offsetWidth
      canvas.height = canvas.offsetHeight
    })
    ro.observe(canvas)
    canvas.width = canvas.offsetWidth
    canvas.height = canvas.offsetHeight

    // Center camera on data
    if (stars.length > 0) {
      const cx = stars.reduce((s, st) => s + st.x, 0) / stars.length
      const cy = stars.reduce((s, st) => s + st.y, 0) / stars.length
      cameraRef.current = { x: cx, y: cy, zoom: 0.4 }
    }

    return () => ro.disconnect()
  }, [stars])

  // Animation loop
  useEffect(() => {
    rafRef.current = requestAnimationFrame(draw)
    return () => cancelAnimationFrame(rafRef.current)
  }, [draw])

  // Mouse events
  const onMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const canvas = canvasRef.current
      if (!canvas) return
      const rect = canvas.getBoundingClientRect()
      const mx = e.clientX - rect.left
      const my = e.clientY - rect.top
      setMousePos({ x: mx, y: my })

      if (dragRef.current.dragging) {
        const cam = cameraRef.current
        cameraRef.current = {
          ...cam,
          x: cam.x - (mx - dragRef.current.lastX) / cam.zoom,
          y: cam.y - (my - dragRef.current.lastY) / cam.zoom,
        }
        dragRef.current.lastX = mx
        dragRef.current.lastY = my
        return
      }

      // Hover detection
      const cam = cameraRef.current
      const { width, height } = canvas
      const toWorld = (sx: number, sy: number) => ({
        wx: (sx - width / 2) / cam.zoom + cam.x,
        wy: (sy - height / 2) / cam.zoom + cam.y,
      })
      const { wx, wy } = toWorld(mx, my)

      let closest: Star | null = null
      let minDist = 20 / cam.zoom
      for (const star of filteredStars) {
        const d = Math.hypot(star.x - wx, star.y - wy)
        if (d < minDist) {
          minDist = d
          closest = star
        }
      }
      setHovered(closest)
    },
    [filteredStars]
  )

  const onMouseDown = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    dragRef.current = { dragging: true, lastX: e.clientX, lastY: e.clientY }
  }, [])

  const onMouseUp = useCallback(() => {
    dragRef.current.dragging = false
  }, [])

  const onWheel = useCallback((e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault()
    const factor = e.deltaY < 0 ? 1.1 : 0.9
    cameraRef.current = {
      ...cameraRef.current,
      zoom: Math.min(5, Math.max(0.05, cameraRef.current.zoom * factor)),
    }
  }, [])

  const constellationNames = Object.keys(clusters)

  return (
    <div className="galaxy-viewer-wrap">
      {/* Filter chips */}
      <div className="galaxy-filters">
        <button
          className={`galaxy-chip ${activeFilter === null ? 'active' : ''}`}
          onClick={() => setActiveFilter(null)}
        >
          All
        </button>
        {constellationNames.map((name) => (
          <button
            key={name}
            className={`galaxy-chip ${activeFilter === name ? 'active' : ''}`}
            onClick={() => setActiveFilter(activeFilter === name ? null : name)}
          >
            {name}
          </button>
        ))}
      </div>

      {/* Canvas */}
      <canvas
        ref={canvasRef}
        className="galaxy-canvas"
        onMouseMove={onMouseMove}
        onMouseDown={onMouseDown}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onWheel={onWheel}
        style={{ cursor: dragRef.current.dragging ? 'grabbing' : hovered ? 'pointer' : 'grab' }}
      />

      {/* Legend */}
      <div className="galaxy-legend">
        <span className="legend-item">
          <span className="dot" style={{ background: '#ffe66d' }} /> Broadcast
        </span>
        <span className="legend-item">
          <span className="dot" style={{ background: '#663399', opacity: 0.6 }} /> Trace
        </span>
        <span className="legend-item">
          <span className="dot" style={{ background: '#00f0ff', height: 2, width: 16 }} /> Knowledge
        </span>
        <span className="legend-stats">
          {filteredStars.length} stars · {edges.length} links · {nebulae.length} nebulae
        </span>
      </div>

      <style jsx>{`
        .galaxy-viewer-wrap {
          position: relative;
          width: 100%;
          height: 500px;
          display: flex;
          flex-direction: column;
          background: #020210;
          border-radius: 8px;
          overflow: hidden;
          border: 1px solid #1a1a3e;
        }
        .galaxy-filters {
          display: flex;
          gap: 6px;
          padding: 10px 14px;
          flex-wrap: wrap;
          background: rgba(0, 0, 0, 0.4);
          border-bottom: 1px solid #1a1a3e;
          z-index: 2;
        }
        .galaxy-chip {
          background: transparent;
          border: 1px solid #2a2a5e;
          color: #8888cc;
          padding: 3px 10px;
          border-radius: 20px;
          font-size: 11px;
          cursor: pointer;
          font-family: monospace;
          transition: all 0.15s;
        }
        .galaxy-chip:hover {
          border-color: #00f0ff;
          color: #00f0ff;
        }
        .galaxy-chip.active {
          background: rgba(0, 240, 255, 0.1);
          border-color: #00f0ff;
          color: #00f0ff;
        }
        .galaxy-canvas {
          flex: 1;
          width: 100%;
          display: block;
        }
        .galaxy-legend {
          display: flex;
          align-items: center;
          gap: 14px;
          padding: 8px 14px;
          background: rgba(0, 0, 0, 0.5);
          border-top: 1px solid #1a1a3e;
          font-size: 11px;
          color: #666;
          font-family: monospace;
        }
        .legend-item {
          display: flex;
          align-items: center;
          gap: 5px;
        }
        .dot {
          display: inline-block;
          width: 8px;
          height: 8px;
          border-radius: 50%;
        }
        .legend-stats {
          margin-left: auto;
          color: #444;
        }
      `}</style>
    </div>
  )
}
