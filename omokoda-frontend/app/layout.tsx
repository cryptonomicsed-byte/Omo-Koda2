import type { Metadata } from 'next'
import './globals.css'
import { Header } from '@/components/layout/Header'
import { Sidebar } from '@/components/layout/Sidebar'
import { CommandPalette } from '@/components/layout/CommandPalette'

export const metadata: Metadata = {
  title: 'Omo-Koda — Sovereign Agent OS',
  description: 'Decentralized AI Agent Platform governed by the 7 Hermetic Principles',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className="dark">
      <body className="bg-black text-white min-h-screen flex flex-col">
        <Header />
        <div className="flex flex-1 min-h-0">
          <Sidebar />
          <main className="flex-1 overflow-auto bg-black">
            {children}
          </main>
        </div>
        <CommandPalette />
      </body>
    </html>
  )
}
