"use client";

import {
  ResponsiveContainer,
  AreaChart,
  Area,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
} from "recharts";
import { format, parseISO } from "date-fns";
import type { BurndownDataPoint } from "@/types/api";

interface BurndownChartProps {
  actual: BurndownDataPoint[];
  ideal?: BurndownDataPoint[];
  height?: number;
}

export function BurndownChart({ actual, ideal, height = 350 }: BurndownChartProps) {
  const merged = actual.map((a) => {
    const idealPoint = ideal?.find((i) => i.date === a.date);
    return {
      date: format(parseISO(a.date), "MMM d"),
      remaining: a.remaining,
      ideal: idealPoint?.remaining,
    };
  });

  return (
    <ResponsiveContainer width="100%" height={height}>
      <AreaChart data={merged} margin={{ top: 5, right: 20, left: 0, bottom: 5 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
        <XAxis dataKey="date" tick={{ fontSize: 12 }} stroke="#9ca3af" />
        <YAxis tick={{ fontSize: 12 }} stroke="#9ca3af" />
        <Tooltip
          contentStyle={{
            backgroundColor: "white",
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
            fontSize: "12px",
          }}
        />
        <Legend />
        <Area
          type="monotone"
          dataKey="remaining"
          stroke="#3b82f6"
          fill="#93c5fd"
          fillOpacity={0.3}
          strokeWidth={2}
          name="Actual"
        />
        {ideal && (
          <Line
            type="monotone"
            dataKey="ideal"
            stroke="#9ca3af"
            strokeWidth={2}
            strokeDasharray="5 5"
            dot={false}
            name="Ideal"
          />
        )}
      </AreaChart>
    </ResponsiveContainer>
  );
}
