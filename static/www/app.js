const video = document.getElementById("webcam");
const nalCountDisplay = document.getElementById("nalCount");
const ws = new WebSocket("ws://127.0.0.1:8080/ws");

navigator.mediaDevices.getUserMedia({
    video: {
        aspectRatio: 16 / 9,
        width: { ideal: 1280 },
        height: { ideal: 720 }
    }
})
    .then((stream) => {
        video.srcObject = stream;

        const mediaRecorder = new MediaRecorder(stream, {});
        mediaRecorder.ondataavailable = (event) => {
            if (event.data.size > 0) {
                ws.send(event.data);
            }
        };
        mediaRecorder.start(100);
    })
    .catch((err) => console.error("Error accessing webcam:", err));

ws.onmessage = (event) => {
    nalCountDisplay.textContent = event.data;
};

