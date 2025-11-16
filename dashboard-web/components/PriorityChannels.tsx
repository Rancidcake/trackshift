'use client'

import { useState } from 'react'
import { AlertTriangle, ArrowUp, ArrowRight, ArrowDown, Settings } from 'lucide-react'

type Priority = 'urgent' | 'high' | 'normal' | 'bulk'

interface PriorityQueue {
  priority: Priority
  name: string
  count: number
  color: string
  icon: React.ReactNode
}

export default function PriorityChannels() {
  const [queues, setQueues] = useState<PriorityQueue[]>([
    { priority: 'urgent', name: 'Urgent', count: 3, color: 'red', icon: <AlertTriangle className="w-4 h-4" /> },
    { priority: 'high', name: 'High', count: 12, color: 'yellow', icon: <ArrowUp className="w-4 h-4" /> },
    { priority: 'normal', name: 'Normal', count: 45, color: 'green', icon: <ArrowRight className="w-4 h-4" /> },
    { priority: 'bulk', name: 'Bulk', count: 128, color: 'green', icon: <ArrowDown className="w-4 h-4" /> },
  ])

  const [selectedPriority, setSelectedPriority] = useState<Priority | null>(null)

  const getColorClasses = (priority: Priority) => {
    switch (priority) {
      case 'urgent':
        return 'border-red-500 bg-red-500/10 text-red-400'
      case 'high':
        return 'border-yellow-500 bg-yellow-500/10 text-yellow-400'
      case 'normal':
        return 'border-primary-green bg-primary-green/10 text-primary-green'
      case 'bulk':
        return 'border-primary-green bg-primary-green/10 text-primary-green'
      default:
        return 'border-gray-500 bg-gray-500/10 text-gray-400'
    }
  }

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2">
          <Settings className="w-6 h-6" />
          Priority Channels
        </h2>
      </div>

      {/* Priority Queues */}
      <div className="space-y-3 mb-6">
        {queues.map((queue) => (
          <div
            key={queue.priority}
            className={`glass-card p-4 border-2 ${getColorClasses(queue.priority)} cursor-pointer transition-all duration-300 hover:scale-105 ${
              selectedPriority === queue.priority ? 'ring-2 ring-primary-green' : ''
            }`}
            onClick={() => setSelectedPriority(
              selectedPriority === queue.priority ? null : queue.priority
            )}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={getColorClasses(queue.priority)}>
                  {queue.icon}
                </div>
                <div>
                  <div className="font-bold">{queue.name}</div>
                  <div className="text-xs text-gray-400">Priority Level</div>
                </div>
              </div>
              <div className="text-2xl font-bold">{queue.count}</div>
            </div>
          </div>
        ))}
      </div>

      {/* Priority Assignment Controls */}
      <div className="space-y-3">
        <div className="text-sm text-gray-400 mb-2">Assign Priority</div>
        <div className="grid grid-cols-2 gap-2">
          {(['urgent', 'high', 'normal', 'bulk'] as Priority[]).map((priority) => (
            <button
              key={priority}
              className={`px-3 py-2 rounded-lg border text-sm font-medium transition-all duration-300 ${
                getColorClasses(priority)
              } hover:scale-105`}
            >
              {priority.charAt(0).toUpperCase() + priority.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Selected Priority Info */}
      {selectedPriority && (
        <div className="mt-4 p-4 glass-card border border-dark-border">
          <div className="text-sm text-gray-400 mb-2">Queue Details</div>
          <div className="text-lg font-bold text-primary-green">
            {queues.find(q => q.priority === selectedPriority)?.count} items in queue
          </div>
        </div>
      )}
    </div>
  )
}

