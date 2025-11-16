'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { Server, Upload } from 'lucide-react'

export default function HomePage() {
  const router = useRouter()

  useEffect(() => {
    // Redirect to client dashboard by default
    router.push('/client')
  }, [router])

  return (
    <div className="min-h-screen bg-black text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-8">PitlinkPQC</h1>
        <div className="grid grid-cols-2 gap-6">
          <a
            href="/client"
            className="bg-gray-900 border border-gray-800 rounded-lg p-8 hover:border-primary-green transition-colors"
          >
            <Upload className="w-12 h-12 text-primary-green mx-auto mb-4" />
            <h2 className="text-xl font-bold mb-2">Client Dashboard</h2>
            <p className="text-gray-400">File transfers and uploads</p>
          </a>
          <a
            href="/server"
            className="bg-gray-900 border border-gray-800 rounded-lg p-8 hover:border-primary-green transition-colors"
          >
            <Server className="w-12 h-12 text-primary-green mx-auto mb-4" />
            <h2 className="text-xl font-bold mb-2">Server Dashboard</h2>
            <p className="text-gray-400">Server monitoring and health</p>
          </a>
        </div>
      </div>
    </div>
  )
}
