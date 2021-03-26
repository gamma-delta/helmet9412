//! Has some standard patterns and stuff

pub const PATTERNS: &[&str] = &[
    // Gentle random noise from tixy
    "cos(t+i+x*y)",
    // A spiral
    "let a=x-16;let b=y-16;max(sin(sqrt(a*a+b*b)+atan2(b,a)-2*t),0.)",
    // Waves from tixy
    "sin(x/2)-sin(x-t)-y+6",
    // Using the accumulator
    "a+(rand()-0.5)*(i/(32*32))",
    // Cool fire
    "max((y/40+v*2)-tan(x/PI+rand()-t+a)**2,0.)",
    // Voice ring
    "let c=x-16;let d=y-16;let h=sqrt(c*c+d*d);sin(1+h/(8+2*v+rand()))**20-abs(y/60*sin(t/5)+h/100)",
];
