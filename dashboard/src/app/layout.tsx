import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { RepoProvider } from "@/components/layout/repo-provider";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "greport - GitHub Repository Analytics",
  description:
    "Interactive dashboard for GitHub repository reporting and analytics",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <RepoProvider>
          <Sidebar />
          <div className="lg:pl-60">
            <Header />
            <main className="px-4 py-6 sm:px-6 lg:px-8">{children}</main>
          </div>
        </RepoProvider>
      </body>
    </html>
  );
}
