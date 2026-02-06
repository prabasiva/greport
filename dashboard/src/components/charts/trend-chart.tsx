"use client";

import {
  ResponsiveContainer,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
} from "recharts";
import { format, parseISO } from "date-fns";

interface TrendChartProps {
  data: { period_start: string; opened: number; closed: number }[];
  height?: number;
}

export function TrendChart({ data, height = 300 }: TrendChartProps) {
  const chartData = data.map((d) => ({
    ...d,
    label: format(parseISO(d.period_start), "MMM d"),
  }));

  return (
    <ResponsiveContainer width="100%" height={height}>
      <LineChart data={chartData} margin={{ top: 5, right: 20, left: 0, bottom: 5 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
        <XAxis dataKey="label" tick={{ fontSize: 12 }} stroke="#9ca3af" />
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
        <Line
          type="monotone"
          dataKey="opened"
          stroke="#ef4444"
          strokeWidth={2}
          dot={{ r: 3 }}
          name="Opened"
        />
        <Line
          type="monotone"
          dataKey="closed"
          stroke="#22c55e"
          strokeWidth={2}
          dot={{ r: 3 }}
          name="Closed"
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
