class NalUnitTable {
    nalUnits = {};
    element;

    constructor(element) {
        this.element = element;
    }

    render() {
        const nalUnitsArray = Object.entries(this.nalUnits);
        this.element.innerHTML = "";
        for (let [key, value] of nalUnitsArray) {
            this.element.innerHTML += `
                <tr>
                    <td>${key}</td>
                    <td>${value}</td>
                </tr>
            `;
        }
    }

    update(nalUnit) {
        if (this.nalUnits[`${nalUnit} `]) this.nalUnits[`${nalUnit} `] += 1;
        else this.nalUnits[`${nalUnit} `] = 1;
        this.render();
    }
}

const video = document.getElementById("webcam");
const table = new NalUnitTable(document.getElementById("nal-unit-table"));
const ws = new WebSocket("ws://127.0.0.1:8080/parser");

navigator.mediaDevices.getUserMedia({
    video: {
        aspectRatio: 16 / 9,
        width: { ideal: 1280 },
        height: { ideal: 720 }
    }
})
    .then((stream) => {
        video.srcObject = stream;

        const mediaRecorder = new MediaRecorder(stream, { mimeType: 'video/webm; codecs=h264' });
        mediaRecorder.ondataavailable = (event) => {
            if (event.data.size > 0) {
                ws.send(event.data);
            }
        };
        mediaRecorder.start(100);
    })
    .catch((err) => console.error("Error accessing webcam:", err));

ws.onmessage = (event) => {
    table.update(event.data)
};

