// Component tests render with the app's own stylesheet, so they see what the person sees.
import '../../app.css';

// Tests check where a state ends up (hovered, focused), not the animation on the way there.
const settled = document.createElement('style');
settled.textContent = '*, *::before, *::after { transition: none !important; animation-duration: 0s !important; }';
document.head.append(settled);
