// WebSocket connection
let ws = null;
let isConnected = false;

// Mouse movement sensitivity factor (will be loaded from localStorage or use default)
let MOVE_FACTOR = 1.8;

// Scroll accumulator for smooth scrolling
let scrollAccumulator = 0;
const SCROLL_THRESHOLD = 20; // pixels to accumulate before sending scroll command

// Last position tracking for smooth movement
let lastPanDelta = { x: 0, y: 0 };

// Two finger tap tracking (to prevent single tap after two finger tap)
let lastTwoFingerTapTime = 0;
const TWO_FINGER_TAP_BLOCK_DURATION = 500; // ms

// Custom double tap tracking for instant response
let lastTapTime = 0;
let tapTimeout = null;
const DOUBLE_TAP_INTERVAL = 180; // ms - reduced for faster single tap response

// Initialize when page loads
document.addEventListener('DOMContentLoaded', () => {
    loadSettings();
    initWebSocket();
    initTouchpad();
    initTextInput();
    initFunctionKeys();
    initSensitivityControls();
});

// WebSocket initialization
function initWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    console.log('Connecting to:', wsUrl);
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        console.log('WebSocket connected');
        isConnected = true;
        updateStatus('Connected', true);
    };

    ws.onclose = () => {
        console.log('WebSocket disconnected');
        isConnected = false;
        updateStatus('Disconnected', false);

        // Attempt to reconnect after 3 seconds
        setTimeout(() => {
            updateStatus('Connecting', false);
            initWebSocket();
        }, 3000);
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateStatus('Error', false);
    };

    ws.onmessage = (event) => {
        console.log('Message from server:', event.data);
    };
}

// Update connection status display
function updateStatus(text, connected) {
    const statusText = document.getElementById('status-text');
    const statusIndicator = document.getElementById('status-indicator');

    statusText.textContent = text;

    if (connected) {
        statusIndicator.classList.remove('disconnected');
        statusIndicator.classList.add('connected');
    } else {
        statusIndicator.classList.remove('connected');
        statusIndicator.classList.add('disconnected');
    }
}

// Send message via WebSocket
function sendMessage(msg) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
        return true;
    } else {
        console.warn('WebSocket not ready, message not sent:', msg);
        return false;
    }
}

// Initialize touchpad with Hammer.js
function initTouchpad() {
    const touchpad = document.getElementById('touchpad');

    // Create Hammer instance
    const hammer = new Hammer.Manager(touchpad, {
        touchAction: 'none',
        recognizers: [
            // Pan recognizer (for movement and scroll)
            [Hammer.Pan, {
                direction: Hammer.DIRECTION_ALL,
                threshold: 5, // Small threshold to distinguish from tap
                pointers: 0 // Accept any number of pointers
            }],
            // Tap recognizer (we handle double tap manually for instant response)
            [Hammer.Tap, {
                event: 'tap',
                pointers: 1,
                taps: 1,
                time: 200,
                threshold: 15
            }],
            // Two finger tap (for right click)
            [Hammer.Tap, {
                event: 'twofingertap',
                pointers: 2,
                taps: 1,
                time: 200,
                threshold: 15
            }]
        ]
    });

    // Track whether we're in a pan gesture
    let isPanning = false;
    let panPointerCount = 0;

    // Pan start - track pointer count
    hammer.on('panstart', (e) => {
        isPanning = true;
        panPointerCount = e.pointers.length;
        lastPanDelta = { x: e.deltaX, y: e.deltaY };
        scrollAccumulator = 0; // Reset scroll accumulator
        console.log(`Pan start with ${panPointerCount} finger(s)`);
    });

    // Pan move - mouse movement or scroll
    hammer.on('panmove', (e) => {
        if (!isPanning) return;

        // Calculate incremental delta (difference from last position)
        const dx = e.deltaX - lastPanDelta.x;
        const dy = e.deltaY - lastPanDelta.y;
        lastPanDelta = { x: e.deltaX, y: e.deltaY };

        if (panPointerCount === 1) {
            // Single finger - mouse movement
            const moveX = Math.round(dx * MOVE_FACTOR);
            const moveY = Math.round(dy * MOVE_FACTOR);

            if (moveX !== 0 || moveY !== 0) {
                sendMessage(['m', moveX, moveY]);
            }
        } else if (panPointerCount === 2) {
            // Two fingers - scroll wheel with accumulation
            scrollAccumulator += dy;

            // Send scroll command when threshold is reached
            if (Math.abs(scrollAccumulator) >= SCROLL_THRESHOLD) {
                // Calculate how many scroll units to send
                const scrollUnits = Math.floor(Math.abs(scrollAccumulator) / SCROLL_THRESHOLD);
                const scrollDirection = scrollAccumulator > 0 ? -1 : 1; // Invert: down swipe = scroll down

                // Send scroll commands
                for (let i = 0; i < scrollUnits; i++) {
                    sendMessage(['w', scrollDirection]);
                }

                // Reduce accumulator by the amount we've scrolled
                scrollAccumulator = scrollAccumulator % SCROLL_THRESHOLD * Math.sign(scrollAccumulator);
            }
        }
    });

    // Pan end - reset state
    hammer.on('panend pancancel', (e) => {
        isPanning = false;
        panPointerCount = 0;
        lastPanDelta = { x: 0, y: 0 };
        scrollAccumulator = 0; // Reset scroll accumulator
        console.log('Pan end');
    });

    // Custom tap handler with instant double tap detection
    hammer.on('tap', (e) => {
        if (isPanning) return; // Ignore taps during pan

        const now = Date.now();

        // Ignore single tap shortly after two finger tap
        if (now - lastTwoFingerTapTime < TWO_FINGER_TAP_BLOCK_DURATION) {
            console.log('Tap blocked (too soon after two finger tap)');
            return;
        }

        // Check if this is a double tap (instant recognition)
        if (now - lastTapTime < DOUBLE_TAP_INTERVAL) {
            // This is a double tap - fire immediately!
            if (tapTimeout) {
                clearTimeout(tapTimeout);
                tapTimeout = null;
            }
            console.log('Double tap - instant double left click');
            sendMessage(['b', 'l', 2]);
            lastTapTime = 0; // Reset to prevent triple tap
        } else {
            // Potentially a single tap - delay to check for double tap
            if (tapTimeout) {
                clearTimeout(tapTimeout);
            }
            tapTimeout = setTimeout(() => {
                console.log('Single tap - left click');
                sendMessage(['b', 'l', 1]);
                tapTimeout = null;
            }, DOUBLE_TAP_INTERVAL);
            lastTapTime = now;
        }
    });

    // Two finger tap - right click
    hammer.on('twofingertap', (e) => {
        if (isPanning) return;

        const now = Date.now();
        lastTwoFingerTapTime = now;

        console.log('Two finger tap - right click');
        sendMessage(['b', 'r', 1]);
    });

    // Prevent context menu
    touchpad.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });
}

// Initialize text input
function initTextInput() {
    const textInput = document.getElementById('textInput');
    const btnSendText = document.getElementById('btn-send-text');

    const sendText = (appendEnter = false) => {
        const text = textInput.value;

        if (text.trim() !== '') {
            // Append Enter key only if requested (from soft keyboard Enter)
            const messageText = appendEnter ? text + '\n' : text;

            if (sendMessage(['t', messageText])) {
                // Clear input after successful send
                textInput.value = '';
                // Close soft keyboard by removing focus
                textInput.blur();
                // Visual feedback
                btnSendText.classList.add('scale-95');
                setTimeout(() => {
                    btnSendText.classList.remove('scale-95');
                }, 150);
            } else {
                alert('Not connected to server, cannot send text');
            }
        }
    };

    // Send button - text only, no Enter key
    btnSendText.addEventListener('click', () => sendText(false));

    // Soft keyboard Enter key - text + Enter key
    textInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            sendText(true);
        }
    });

    // Prevent touch events from propagating
    btnSendText.addEventListener('touchstart', (e) => {
        e.preventDefault();
        btnSendText.click();
    });
}

// Initialize function keys
function initFunctionKeys() {
    const keyButtons = document.querySelectorAll('[data-key]');

    keyButtons.forEach(button => {
        const sendKey = () => {
            const keyName = button.getAttribute('data-key');
            if (sendMessage(['k', keyName])) {
                // Visual feedback
                button.classList.add('scale-95');
                setTimeout(() => {
                    button.classList.remove('scale-95');
                }, 150);
                console.log('Function key pressed:', keyName);
            } else {
                console.warn('Not connected, cannot send key:', keyName);
            }
        };

        // Handle click events
        button.addEventListener('click', sendKey);

        // Handle touch events (prevent default to avoid double firing)
        button.addEventListener('touchstart', (e) => {
            e.preventDefault();
            sendKey();
        });
    });
}

// Load settings from localStorage
function loadSettings() {
    const savedMoveFactor = localStorage.getItem('moveFactor');

    if (savedMoveFactor !== null) {
        MOVE_FACTOR = parseFloat(savedMoveFactor);
    }

    console.log('Loaded settings:', { MOVE_FACTOR });
}

// Save settings to localStorage
function saveSettings() {
    localStorage.setItem('moveFactor', MOVE_FACTOR.toString());
    console.log('Saved settings:', { MOVE_FACTOR });
}

// Initialize sensitivity controls
function initSensitivityControls() {
    const moveFactorSlider = document.getElementById('move-factor');
    const moveFactorValue = document.getElementById('move-factor-value');

    // Set initial values from loaded settings
    moveFactorSlider.value = MOVE_FACTOR;
    moveFactorValue.textContent = MOVE_FACTOR.toFixed(1) + 'x';

    // Mouse movement sensitivity
    moveFactorSlider.addEventListener('input', (e) => {
        MOVE_FACTOR = parseFloat(e.target.value);
        moveFactorValue.textContent = MOVE_FACTOR.toFixed(1) + 'x';
        saveSettings();
    });
}
