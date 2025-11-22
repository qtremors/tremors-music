interface Props {
  value: number;
  max: number;
  onChange: (val: number) => void;
  className?: string;
}

export function SquigglyProgress({ value, max, onChange, className = "" }: Props) {
  const percentage = max > 0 ? (value / max) * 100 : 0;
  
  return (
    <div className={`relative h-10 flex items-center group cursor-pointer ${className}`}
      onClick={(e) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const p = x / rect.width;
        onChange(p * max);
      }}
    >
      {/* Track (Straight Line) */}
      <div className="absolute inset-x-0 h-[2px] bg-white/20 rounded-full" />

      {/* Active (Squiggly Line) */}
      {/* Use a CSS mask or SVG pattern to create the squiggle */}
      <div 
        className="absolute left-0 h-[6px] bg-apple-accent top-1/2 -translate-y-1/2 overflow-hidden transition-all duration-100"
        style={{ width: `${percentage}%` }}
      >
        <svg 
          width="100%" 
          height="100%" 
          preserveAspectRatio="none"
          className="absolute inset-0 w-full h-full"
        >
          <path 
            d="M0 3 Q 5 0, 10 3 T 20 3 T 30 3 T 40 3 T 50 3 T 60 3 T 70 3 T 80 3 T 90 3 T 100 3" 
            fill="none" 
            stroke="currentColor" 
            strokeWidth="6"
            vectorEffect="non-scaling-stroke"
            className="animate-pulse"
          />
        </svg>
      </div>

      {/* Thumb (Knob) */}
      <div 
        className="absolute top-1/2 -translate-y-1/2 w-4 h-4 bg-white rounded-full shadow-lg transition-transform scale-0 group-hover:scale-100"
        style={{ left: `${percentage}%`, transform: 'translate(-50%, -50%)' }}
      />
      
      {/* Input for dragging (invisible but functional) */}
      <input
        type="range"
        min={0}
        max={max}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-20"
      />
    </div>
  );
}