'use client'

import { useEffect, useState } from 'react'

interface StreamingTextProps {
  text: string
  speed?: number
  className?: string
  showCursor?: boolean
}

export function StreamingText({
  text,
  speed = 20,
  className = '',
  showCursor = true,
}: StreamingTextProps) {
  const [displayed, setDisplayed] = useState('')
  const [done, setDone] = useState(false)

  useEffect(() => {
    setDisplayed('')
    setDone(false)
    if (!text) return

    let i = 0
    const interval = setInterval(() => {
      if (i < text.length) {
        setDisplayed(text.slice(0, i + 1))
        i++
      } else {
        setDone(true)
        clearInterval(interval)
      }
    }, speed)

    return () => clearInterval(interval)
  }, [text, speed])

  return (
    <span className={className}>
      {displayed}
      {showCursor && !done && (
        <span className="inline-block w-0.5 h-4 bg-brand-400 animate-cursor-blink ml-0.5 align-middle" />
      )}
    </span>
  )
}
