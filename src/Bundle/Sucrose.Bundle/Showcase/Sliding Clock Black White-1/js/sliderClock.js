const strips = [...document.querySelectorAll(".strip")];
const numberSize = "4"; // in rem

var lastTime = new Array(-1, -1, -1)

// highlight number i on strip s for 1 second
function highlight(strip, d) {
	strips[strip]
		.querySelector(`.number:nth-of-type(${d + 1})`)
		.classList.add("pop");

	setTimeout(() => {
		strips[strip]
			.querySelector(`.number:nth-of-type(${d + 1})`)
			.classList.remove("pop");
	}, 950); // causes ticking
}

function stripSlider(strip, id, number) {
	let d1 = Math.floor(number / 10);
	let d2 = number % 10;

	if (lastTime[id] == -1 || lastTime[id] != number) {
		strips[strip].style.transform = `translateY(${d1 * -numberSize}rem)`;
		strips[strip + 1].style.transform = `translateY(${d2 * -numberSize}rem)`;

		lastTime[id] = number;
	}

	highlight(strip, d1);
	highlight(strip + 1, d2);
}

function updateClock() {
	// get new time
	const time = new Date();

	// get h,m,s
	const hours = time.getHours();
	const mins = time.getMinutes();
	const secs = time.getSeconds();

	// slide strips
	stripSlider(0, 0, hours);
	stripSlider(2, 1, mins);
	stripSlider(4, 2, secs);
}

// set Timer for clock-update
setInterval(updateClock, 1000);

updateClock();

function changeCSS() { //更换颜色
	var obj = document.getElementById("cssstyle");
	console.log(obj);
	var box = document.getElementById("changeButton");

	// currentStyle: IE下获取元素样式的值
	if (box.currentStyle) {
		console.log('this is IE.');
		console.log(box.currentStyle.backgroundColor);
		var color = box.currentStyle.backgroundColor;
	} else {
		// chorme and firefox
		console.log('this is Chrome and firefox.');
		console.log(document.defaultView.getComputedStyle(box, false).backgroundColor);
		var color = document.defaultView.getComputedStyle(box, false).backgroundColor;
	}
	if (color == 'rgb(44, 44, 44)') { //根据颜色判断更换哪种样式
		console.log("ChangeStyle1");
		obj.setAttribute("href", "./css/dark.css");
	} else {
		console.log("ChangeStyle2");
		obj.setAttribute("href", "./css/light.css");
	}
}

function themeMode(index) {
	var obj = document.getElementById("cssstyle");
	console.log(obj);

	if (index == 0) {
		console.log("ChangeStyle1");
		obj.setAttribute("href", "./css/dark.css");
	} else {
		console.log("ChangeStyle2");
		obj.setAttribute("href", "./css/light.css");
	}
}

function themeChange(visible) {
	if (visible) {
		$('#changeButton').show();
	} else {
		$('#changeButton').hide();
	}
}

var now = new Date;
console.log(now.getFullYear());
var weekList = new Array("Sun", "Mon", "Tue", "Wed", "Thur", "Fri", "Sat");
var monthList = new Array("January", "February", "March", "April", "May", "June",
	"July", "August", "September", "October", "November", "December");

var week = weekList[now.getDay()];
var month = monthList[now.getMonth()];
var day = now.getDate();
var year = now.getFullYear();
if (day >= 1 && day <= 9) {
	day = "0" + day;
}
var date = month + " " + day + "," + year;
document.getElementsByClassName("date")[0].innerHTML = date;
document.getElementsByClassName("week")[0].innerHTML = week + '.';
$('.datetext').text(month + ". " + day + " " + week);