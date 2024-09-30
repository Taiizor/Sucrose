var amdGpu = [];
var intelGpu = [];
var nvidiaGpu = [];
var cpuCounter = 0;
var gpuCounter = 0;
var netDownCounter = 0;
var netUpCounter = 0;
var memUsed = 0;
var cpuName = "";
var gpuName = "";
var memoryName = "";
var netCardName = "";
var isChartInit = false;
var cpu = false;
var gpu = false;
var mem = false;
var net = false;

var chartColors = {
	red: 'rgb(255, 99, 132)',
	orange: 'rgb(255, 159, 64)',
	yellow: 'rgb(255, 205, 86)',
	green: 'rgb(0, 192, 0)',
	blue: 'rgb(54, 162, 235)',
	purple: 'rgb(153, 102, 255)',
	grey: 'rgb(201, 203, 207)',
	lightGrey: 'rgb(105, 105, 105)',
	black: 'rgb(0, 0, 0)',
};
var color = Chart.helpers.color;

//global chart defaults
Chart.defaults.global.title.display = true;
Chart.defaults.global.legend.display = false;
Chart.defaults.global.responsive = true;
Chart.defaults.global.elements.pointRadius = 0;
Chart.defaults.global.tooltips.enabled = false;

var cpuChartConfig = {
	type: 'line',
	data: {
		datasets: [{
			label: 'Dataset 1',
			backgroundColor: color(chartColors.grey).alpha(0.5).rgbString(),
			borderColor: chartColors.grey,
			fill: false,
			lineTension: 0,
			borderDash: [0, 0],
			pointRadius: 0,
			data: []
		}],
	},
	options: {
		maintainAspectRatio: false,
		legend: {
			display: false,
		},
		title: {
			display: true,
			text: 'Processor',
		},
		scales: {
			xAxes: [{
				type: 'realtime',
				realtime: {
					duration: 20000,
					refresh: 1000,
					delay: 1000,
					onRefresh: onRefresh
				},
				//gridLines: { color: "#484848" },
				ticks: {
					display: false,
				},
			}],
			yAxes: [{
				scaleLabel: {
					display: false,
					labelString: '%'
				},
				//gridLines: { color: "#484848" },
				ticks: {
					beginAtZero: true,
					max: 100
				}
			}]
		},
	}
};

var gpuChartConfig = {
	type: 'line',
	data: {
		datasets: [{
			label: 'Dataset 1 (linear interpolation)',
			backgroundColor: color(chartColors.grey).alpha(0.5).rgbString(),
			borderColor: chartColors.grey,
			fill: false,
			lineTension: 0,
			borderDash: [0, 0],
			pointRadius: 0,
			data: []
		}]
	},
	options: {
		maintainAspectRatio: false,
		legend: {
			display: false,
		},
		title: {
			display: true,
			text: 'Graphics',
		},
		scales: {
			xAxes: [{
				type: 'realtime',
				realtime: {
					duration: 20000,
					refresh: 1000,
					delay: 1000,
					onRefresh: onRefresh
				},
				ticks: {
					display: false,
				},
			}],
			yAxes: [{
				scaleLabel: {
					display: false,
					labelString: '%'
				},
				ticks: {
					beginAtZero: true,
					max: 100
				}
			}]
		},
	}
};

var netChartConfig = {
	type: 'line',
	data: {
		datasets: [{
				label: 'Net Down',
				backgroundColor: color(chartColors.grey).alpha(0.5).rgbString(),
				borderColor: chartColors.grey,
				fill: false,
				lineTension: 0,
				borderDash: [0, 0],
				pointRadius: 0,
				data: []
			},
			{
				label: 'Net Up',
				backgroundColor: color(chartColors.lightGrey).alpha(0.5).rgbString(),
				borderColor: chartColors.lightGrey,
				fill: false,
				lineTension: 0,
				borderDash: [0, 0],
				pointRadius: 0,
				data: []
			}
		]
	},
	options: {
		legend: {
			display: false,
		},
		title: {
			display: true,
			text: 'Network',
		},
		scales: {
			xAxes: [{
				type: 'realtime',
				realtime: {
					duration: 20000,
					refresh: 1000,
					delay: 1000,
					onRefresh: onRefresh
				},
				ticks: {
					display: false,
				},
			}],
			yAxes: [{
				scaleLabel: {
					display: false,
					labelString: '%'
				},
				ticks: {
					beginAtZero: true,
					suggestedMax: 100
				}
			}]
		},
	}
};

var ramChartConfig = {
	type: 'line',
	data: {
		datasets: [{
			label: 'Dataset 1 (linear interpolation)',
			backgroundColor: color(chartColors.grey).alpha(0.5).rgbString(),
			borderColor: chartColors.grey,
			fill: false,
			lineTension: 0,
			borderDash: [0, 0],
			pointRadius: 0,
			data: []
		}]
	},
	options: {
		maintainAspectRatio: false,
		legend: {
			display: false,
		},
		title: {
			display: true,
			text: memoryName,
		},
		scales: {
			xAxes: [{
				type: 'realtime',
				realtime: {
					duration: 20000,
					refresh: 1000,
					delay: 1000,
					onRefresh: onRefresh
				},
				ticks: {
					display: false,
				},
			}],
			yAxes: [{
				scaleLabel: {
					display: false,
					labelString: '%'
				},
				ticks: {
					beginAtZero: true,
					max: 100,
				}
			}]
		},
	}
};

function onRefresh(chart) {
	var data = [];
	switch (chart) {
		case cpuChart:
			data[0] = cpuCounter;
			break;
		case gpuChart:
			data[0] = gpuCounter;
			break;
		case netChart:
			data[0] = netDownCounter;
			data[1] = netUpCounter;
			break;
		case ramChart:
			data[0] = memUsed;
			break;
	}

	var i = 0;
	chart.config.data.datasets.forEach(
		function(dataset) {
			dataset.data.push({
				x: Date.now(),
				y: data[i],
			});
			i++;
		});
}

var cpuChart, gpuChart, netChart, ramChart;

function initChart() {
	cpuChartConfig.options.title.text = cpuName;
	gpuChartConfig.options.title.text = gpuName;
	netChartConfig.options.title.text = netCardName;
	ramChartConfig.options.title.text = memoryName;

	var ctxCpu = document.getElementById('cpuChart').getContext('2d');
	cpuChart = new Chart(ctxCpu, cpuChartConfig);

	var ctxGpu = document.getElementById('gpuChart').getContext('2d');
	gpuChart = new Chart(ctxGpu, gpuChartConfig);

	var ctxNet = document.getElementById('netChart').getContext('2d');
	netChart = new Chart(ctxNet, netChartConfig);

	var ctxRam = document.getElementById('ramChart').getContext('2d');
	ramChart = new Chart(ctxRam, ramChartConfig);
};

function SucroseCpuData(obj) {
	if (obj != null) {
		cpu = true;

		if (obj.State) {
			//hw name
			cpuName = obj.Name;

			//chart data
			cpuCounter = obj.Now;
		}
	}
}

function SucroseGpuData(obj) {
	if (obj != null) {
		gpu = true;

		if (obj.State) {
			amdGpu = obj.Amd;
			intelGpu = obj.Intel;
			nvidiaGpu = obj.Nvidia;
			
			gpuName = obj.Name;

			let highestNow = -Infinity;
			
			let manufacturer = obj.Manufacturer;

			//hw name
			//chart data

			if (manufacturer == "Nvidia" && nvidiaGpu != null) {
				nvidiaGpu.forEach(gpu => {
					if (gpu.Type === "Load" && gpu.Now !== null && gpu.Now > highestNow) {
						highestNow = gpu.Now;
					}
				});

				gpuCounter = highestNow.toFixed(2);
			} else if (manufacturer == "Amd" && amdGpu != null) {
				amdGpu.forEach(gpu => {
					if (gpu.Type === "Load" && gpu.Now !== null && gpu.Now > highestNow) {
						highestNow = gpu.Now;
					}
				});

				gpuCounter = highestNow.toFixed(2);
			} else if (manufacturer == "Intel" && intelGpu != null) {
				intelGpu.forEach(gpu => {
					if (gpu.Type === "Load" && gpu.Now !== null && gpu.Now > highestNow) {
						highestNow = gpu.Now;
					}
				});

				gpuCounter = highestNow.toFixed(2);
			} else {
				gpuName = "Unknown";
				gpuCounter = 0;
			}
		}
	}
}

function SucroseNetworkData(obj) {
	if (obj != null) {
		net = true;

		if (obj.State) {
			//hw name
			netCardName = obj.Name;

			//chart data
			netDownCounter = obj.Download / (1024 * 1024);
			netUpCounter = obj.Upload / (1024 * 1024);
		}
	}
}

function SucroseMemoryData(obj) {
	if (obj != null) {
		mem = true;

		if (obj.State) {
			var totalRam = obj.MemoryAvailable + obj.MemoryUsed;

			//hw name
			memoryName = obj.Name + " (" + totalRam.toFixed(0) + " GB)";

			//chart data
			memUsed = obj.MemoryLoad;
		}
	}
}

function informationCallback() {
	if (!isChartInit && gpu && cpu && mem && net) {
		isChartInit = true;
		initChart();
	}
}

setInterval(informationCallback, 1000);

//initChart();