(function IIFE() {

	let isPlaying = false;
	let isLoop = true;
	let loopOne = false;
	let isShuffle = false;
	let EndTime = 0;
	let Position = 0;
	let timer = null;
	let OldPosition = 0;

	const wheel = document.getElementById("wheel");
	const currentTimeIndicator = document.querySelector(".musicTime__current");
	const leftTimeIndicator = document.querySelector(".musicTime__last");
	const progressBar = document.getElementById("length");
	const player = document.getElementById("player");

	const albumClass = document.getElementById("jsAlbum");
	const cover = document.getElementById("cover");
	const playBtn = document.getElementById("play");
	const loopBtn = document.getElementById("loop");
	const shuffleBtn = document.getElementById("shuffle");
	const forwardBtn = document.getElementById("forward");
	const backwardBtn = document.getElementById("backward");
	const prevBtn = document.getElementById("prev");
	const nextBtn = document.getElementById("next");
	const progressDiv = document.getElementById("progress");
	const container = document.getElementById("container");
	const artist = document.getElementById("artist");
	const title = document.getElementById("title");

	// FORCE PLAY
	function fplay() {
		if (!isPlaying) {
			playBtn.classList.remove("_play");
			playBtn.classList.add("_pause");

			wheel.classList.remove("_play");
			wheel.classList.add("_pause");

			isPlaying = true;
			showTime();
		} else {
			playBtn.classList.remove("_pause");
			playBtn.classList.add("_play");

			wheel.classList.remove("_pause");
			wheel.classList.add("_play");

			isPlaying = false;
			clearInterval(timer);
		}
	}

	// BUTTON PLAY
	function play(e) {
		if (!isPlaying) {
			e.target.classList.remove("_play");
			e.target.classList.add("_pause");

			wheel.classList.remove("_play");
			wheel.classList.add("_pause");

			isPlaying = true;
			showTime();
		} else {
			e.target.classList.remove("_pause");
			e.target.classList.add("_play");

			wheel.classList.remove("_pause");
			wheel.classList.add("_play");

			isPlaying = false;
			clearInterval(timer);
		}
	}

	let timer2 = setInterval(function() {
		if (isPlaying) {
			Position = Position + 1;
		}
	}, 1000);

	// TIME
	function changeBar() {
		let max = EndTime;
		let now = Position;

		const percentage = (now / max).toFixed(3);
		progressBar.style.transition = "";

		//set current time
		const minute = Math.floor(now / 60);
		const second = Math.floor(now % 60);
		const leftTime = max - now;
		currentTimeIndicator.innerHTML = ("0" + minute).substr(-2) + ":" + ("0" + second).substr(-2);

		//set left time
		const leftMinute = Math.floor(leftTime / 60);
		const leftSecond = Math.floor(leftTime % 60);

		leftTimeIndicator.innerHTML = ("0" + leftMinute).substr(-2) + ":" + ("0" + leftSecond).substr(-2);

		//set time bar
		progressBar.style.width = percentage * 100 + "%";
	}

	function showTime() {
		timer = setInterval(() => changeBar(), 500);
	}

	// SWITCHING MUSIC
	function nextMusic(mode) {
		playBtn.classList.remove("_pause");
		playBtn.classList.add("_play");

		wheel.classList.remove("_pause");
		wheel.classList.add("_play");

		isPlaying = false;
		clearInterval(timer);

		if (mode === "next") {
			init();
		} else {
			init();
		}
	}

	// STARTING A RANDOM TRACK
	function shuffle(e) {
		isShuffle = !isShuffle;
		if (isShuffle) {
			e.target.classList.add("_shuffle");
		} else {
			e.target.classList.remove("_shuffle");
		}
	}

	// STOP MUSIC
	function stopMusic() {
		playBtn.classList.add("_play");
		wheel.classList.add("_play");
		isPlaying = false;
	}

	// THE START OF THE NEXT TRACK
	function goToNextMusic() {
		init();
	}

	// THE PLAYBACK MODE OF THE TRACK
	function loop(e) {
		if (!isLoop && !loopOne) {
			isLoop = true;
			loopOne = false;
			e.target.classList.remove("_off");
			e.target.classList.add("_loop");
			audio.loop = false;
			audio.onended = e => goToNextMusic();
		} else if (isLoop && !loopOne) {
			isLoop = true;
			loopOne = true;
			e.target.classList.remove("_loop");
			e.target.classList.add("_repeat");
			audio.loop = true;
			audio.onended = e => goToNextMusic();
		} else {
			isLoop = false;
			loopOne = false;
			e.target.classList.remove("_repeat");
			e.target.classList.add("_off");
			audio.loop = false;
			audio.onended = e => stopMusic();
		}
	}

	// PROGRESS 
	function progress(e) {
		const pos = (e.pageX - progressDiv.getClientRects()[0].x) / progressDiv.getClientRects()[0].width;
	}

	function init() {
		progressBar.style.transition = "none";
		progressBar.style.width = "0%";
	}

	window.SucroseAudioData = function(obj) {
		if (obj.State) {
			title.innerText = obj.Title;
			artist.innerText = obj.Artist;

			if (obj.PlaybackState == "Playing") {
				isPlaying = false;

				var sheet = document.styleSheets[0];
				var rules = sheet.cssRules || sheet.rules;

				for (var i = 0; i < rules.length; i++) {
					var rule = rules[i];
					if (rule.selectorText === '.album::before') {
						if (obj.ThumbnailString == null || obj.ThumbnailString == "") {
							rule.style.backgroundImage = 'url("./../image/no_signal.gif")';
						} else {
							rule.style.backgroundImage = 'url("data:image/png;base64,' + obj.ThumbnailString + '")';
						}
						rule.style.animationPlayState = 'running';
						break;
					}
				}
			} else {
				isPlaying = true;

				var sheet = document.styleSheets[0];
				var rules = sheet.cssRules || sheet.rules;

				for (var i = 0; i < rules.length; i++) {
					var rule = rules[i];
					if (rule.selectorText === '.album::before') {
						if (obj.ThumbnailString == null || obj.ThumbnailString == "") {
							rule.style.backgroundImage = 'url("./../image/no_signal.gif")';
						} else {
							rule.style.backgroundImage = 'url("data:image/png;base64,' + obj.ThumbnailString + '")';
						}
						rule.style.animationPlayState = 'paused';
						break;
					}
				}
			}

			fplay();
		} else {
			var sheet = document.styleSheets[0];
			var rules = sheet.cssRules || sheet.rules;

			for (var i = 0; i < rules.length; i++) {
				var rule = rules[i];
				if (rule.selectorText === '.album::before') {
					rule.style.backgroundImage = 'url("./../image/record.gif")';
					rule.style.animationPlayState = 'paused';
					break;
				}
			}

			title.innerText = "Title";
			artist.innerText = "Artist";

			isPlaying = true;
			fplay();
		}

		EndTime = obj.EndTime / 1000;

		if (OldPosition != obj.Position) {
			OldPosition = obj.Position;
			Position = obj.Position / 1000;
		}
	}

	window.SucroseLoopMode = function(type) {
		if (type == "True") {
			loopBtn.classList.remove("_off");
			loopBtn.classList.add("_loop");
		} else {
			loopBtn.classList.remove("_loop");
			loopBtn.classList.add("_off");
		}
	}

	window.SucroseShuffleMode = function(type) {
		if (type == "True") {
			shuffleBtn.classList.add("_shuffle");
		} else {
			shuffleBtn.classList.remove("_shuffle");
		}
	}

	window.SucrosePropertyListener = function(name, val) {
		switch (name) {
			case "backgroundImage":
				container.style.backgroundImage = 'url("./' + val.folder + '/' + val.value + '")';
				break;
			case "foregroundImage":
				player.style.backgroundImage = 'url("./' + val.folder + '/' + val.value + '")';
				break;
		}
	}

	playBtn.addEventListener("click", play);
	loopBtn.addEventListener("click", loop);

	shuffleBtn.addEventListener("click", shuffle);
	forwardBtn.addEventListener("click", forward);
	backwardBtn.addEventListener("click", backward);

	prevBtn.addEventListener("click", e => nextMusic("prev"));
	nextBtn.addEventListener("click", e => nextMusic("next"));
	progressDiv.addEventListener("click", e => {
		progress(e);
	});

	init();
	fplay();
})();