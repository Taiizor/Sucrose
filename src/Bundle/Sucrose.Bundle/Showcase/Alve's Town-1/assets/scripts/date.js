const dateContainer = document.getElementById("dateContainer");
const dateDay = document.getElementById("dateDay");
const dateMonthDayYear = document.getElementById("dateMonthDayYear");
const dateTime = document.getElementById("dateTime");

function drawDate() {
	dt = new Date();
	dateTime.textContent = dt.toLocaleTimeString(root.locale.locale, {
		hour12: root.hour12,
		hour: "2-digit",
		minute: "2-digit"
	}).substring(0, 5);
	dateMonthDayYear.textContent = dt.toLocaleString(root.locale.locale, {
		month: "long"
	}) + " " + dt.getDate() + ", " + dt.getFullYear();
	dateDay.textContent = dt.toLocaleString(root.locale.locale, {
		weekday: "long"
	});
}

// Function that positions the date container when the Scale property is set
function positionDateContainer(val) {
	// Compensate manual scaling/cropping
	dateContainer.style.paddingTop = (11.5 - (val * 0.225)) + "vw";
	var fontScale = 0.02;
	dateContainer.style.fontSize = (0.8 - (val * fontScale * 0.8)) + "vw";
	dateTime.style.fontSize = (1.9 - (val * fontScale * 1.9)) + "vw";
	dateDay.style.fontSize = (0.6 - (val * fontScale * 0.6)) + "vw";
}