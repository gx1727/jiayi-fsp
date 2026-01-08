import React, { useEffect, useRef, useState } from 'react';
import mermaid from 'mermaid';

// Initialize mermaid
mermaid.initialize({
  startOnLoad: false,
  theme: 'base',
  securityLevel: 'loose',
  fontFamily: 'inherit',
  themeVariables: {
    primaryColor: '#dbeafe',
    primaryTextColor: '#1e3a8a',
    primaryBorderColor: '#2563eb',
    lineColor: '#64748b',
    secondaryColor: '#ede9fe',
    tertiaryColor: '#fce7f3',
  }
});

interface MermaidProps {
  chart: string;
}

const Mermaid: React.FC<MermaidProps> = ({ chart }) => {
  const [svgContent, setSvgContent] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const renderChart = async () => {
      if (!chart) return;
      
      try {
        const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`;
        const { svg } = await mermaid.render(id, chart);
        setSvgContent(svg);
        setError(null);
      } catch (err: any) {
        console.error('Mermaid render error:', err);
        // Don't show the error in UI to keep it clean, just log it. 
        // Or show a fallback. For now, let's show a user-friendly message.
        setError(err.message || 'Failed to render diagram');
      }
    };

    renderChart();
  }, [chart]);

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-700 p-4 rounded text-sm overflow-auto">
        <p className="font-bold">Diagram Error</p>
        <pre className="whitespace-pre-wrap mt-2 text-xs">{error}</pre>
        <pre className="mt-4 bg-gray-100 p-2 rounded text-gray-600 text-xs overflow-x-auto">{chart}</pre>
      </div>
    );
  }

  return (
    <div 
      dangerouslySetInnerHTML={{ __html: svgContent }} 
      className="mermaid-chart flex justify-center w-full"
    />
  );
};

export default Mermaid;
