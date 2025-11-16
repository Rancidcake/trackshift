import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'
import Link from 'next/link'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'PitlinkPQC - Smart File Transfer Dashboard',
  description: 'Real-time monitoring and control for intelligent file transfer system',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <nav className="fixed top-0 left-0 right-0 z-50 bg-dark-bg/80 backdrop-blur-md border-b border-dark-border">
          <div className="container mx-auto px-4 py-3">
            <div className="flex items-center justify-between">
              <Link href="/" className="text-xl font-bold neon-text text-primary-green">
                PitlinkPQC
              </Link>
              <div className="flex items-center gap-4">
                <Link
                  href="/"
                  className="px-3 py-1 rounded-lg text-sm hover:bg-dark-card transition-colors"
                >
                  Home
                </Link>
              </div>
            </div>
          </div>
        </nav>
        <div className="pt-16">
          {children}
        </div>
      </body>
    </html>
  )
}
