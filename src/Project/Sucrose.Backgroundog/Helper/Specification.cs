using LibreHardwareMonitor.Hardware;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Management;
using System.Net;
using System.Text;
using SBEAS = Sucrose.Backgroundog.Extension.AudioSession;
using SBED = Sucrose.Backgroundog.Extension.Data;
using SBEO = Sucrose.Backgroundog.Extension.Object;
using SBER = Sucrose.Backgroundog.Extension.Remote;
using SBEUV = Sucrose.Backgroundog.Extension.UpdateVisitor;
using SBEV = Sucrose.Backgroundog.Extension.Virtual;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SBMM = Sucrose.Backgroundog.Manage.Manager;
using SBSSSS = Sucrose.Backgroundog.Struct.Sensor.SensorStruct;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHN = Skylark.Helper.Numeric;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDSHS = Sucrose.Shared.Dependency.Struct.HostStruct;
using SSEPPE = Skylark.Standard.Extension.Ping.PingExtension;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSIB = Sucrose.Signal.Interface.Backgroundog;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSMMS = Skylark.Struct.Monitor.MonitorStruct;
using SSSHG = Sucrose.Shared.Space.Helper.Graphic;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHF = Skylark.Wing.Helper.Fullscreen;
using SWNM = Skylark.Wing.Native.Methods;
using SWUD = Skylark.Wing.Utility.Desktop;
using SWUP = Skylark.Wing.Utility.Power;
using SWUS = Skylark.Wing.Utility.Screene;
using SystemInformation = System.Windows.Forms.SystemInformation;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Specification
    {
        public static async Task Start()
        {
            if (SBMI.Exit)
            {
                if (SBMI.CpuManagement)
                {
                    SBMI.CpuManagement = false;

                    _ = Task.Run(() =>
                    {
                        try
                        {
                            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

                            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                            {
                                SBMI.CpuData.State = true;
                                SBMI.CpuData.Fullname = SBEO.Check(Object, "Name", string.Empty);
                                SBMI.CpuData.Core = Convert.ToInt32(SBEO.Check(Object, "NumberOfCores", "0"));
                                SBMI.CpuData.Thread = Convert.ToInt32(SBEO.Check(Object, "NumberOfLogicalProcessors", "0"));

                                break;
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.CpuManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.BiosManagement)
                {
                    SBMI.BiosManagement = false;

                    _ = Task.Run(() =>
                    {
                        try
                        {
                            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_BIOS");

                            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                            {
                                SBMI.BiosData.State = true;
                                SBMI.BiosData.Name = SBEO.Check(Object, "Name", string.Empty);
                                SBMI.BiosData.Caption = SBEO.Check(Object, "Caption", string.Empty);
                                SBMI.BiosData.Version = SBEO.Check(Object, "Version", string.Empty);
                                SBMI.BiosData.Description = SBEO.Check(Object, "Description", string.Empty);
                                SBMI.BiosData.ReleaseDate = SBEO.Check(Object, "ReleaseDate", string.Empty);
                                SBMI.BiosData.Manufacturer = SBEO.Check(Object, "Manufacturer", string.Empty);
                                SBMI.BiosData.SerialNumber = SBEO.Check(Object, "SerialNumber", string.Empty);
                                SBMI.BiosData.CurrentLanguage = SBEO.Check(Object, "CurrentLanguage", string.Empty);

                                break;
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.BiosManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                _ = Task.Run(() =>
                {
                    try
                    {
                        DateTime Date = DateTime.Now;

                        SBMI.DateData = new()
                        {
                            State = true,
                            Day = Date.Day,
                            Hour = Date.Hour,
                            Year = Date.Year,
                            Month = Date.Month,
                            Minute = Date.Minute,
                            Second = Date.Second,
                            Millisecond = Date.Millisecond
                        };
                    }
                    catch (Exception Exception)
                    {
                        SSWW.Watch_CatchException(Exception);
                    }
                });

                if (SBMI.AudioManagement)
                {
                    SBMI.AudioManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SBMI.SessionManagement)
                            {
                                SBMI.SessionManagement = false;

                                SBMI.SessionManager = new();

                                SBMI.AudioVisualizer = new();

                                SBMI.AudioVisualizer.AudioDataAvailable += (s, e) =>
                                {
                                    try
                                    {
                                        SBMI.AudioData.Data = e;
                                    }
                                    catch { }
                                };

                                SBMI.AudioVisualizer.Start();

                                SBMI.SessionManager.SessionListChanged += (s, e) => SBEAS.SessionListChanged();
                            }

                            SBEAS.SessionListChanged();

                            await Task.Delay(SBMI.SpecificationTime);

                            SBMI.AudioManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.AudioManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.BatteryManagement)
                {
                    SBMI.BatteryManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.BatteryData.ACPowerStatus = $"{SWUP.GetACPowerStatus()}";
                            SBMI.BatteryData.SavingMode = SWUP.IsBatterySavingMode;
                            SBMI.BatteryData.SaverStatus = $"{SWUP.GetBatterySaverStatus()}";

                            SBMI.BatteryData.LifePercent = SystemInformation.PowerStatus.BatteryLifePercent;
                            SBMI.BatteryData.PowerLineStatus = SystemInformation.PowerStatus.PowerLineStatus;
                            SBMI.BatteryData.FullLifetime = SystemInformation.PowerStatus.BatteryFullLifetime;
                            SBMI.BatteryData.ChargeStatus = SystemInformation.PowerStatus.BatteryChargeStatus;
                            SBMI.BatteryData.LifeRemaining = SystemInformation.PowerStatus.BatteryLifeRemaining;

                            await Task.Delay(SBMI.SpecificationTime);

                            SBMI.BatteryManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.BatteryManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.PingManagement)
                {
                    SBMI.PingManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SSSHN.GetHostEntry())
                            {
                                List<SSDSHS> Hosts = SSSHN.GetHost();

                                foreach (SSDSHS Host in Hosts)
                                {
                                    if (SMMM.PingType == Host.Name)
                                    {
                                        if (string.IsNullOrEmpty(SBMI.PingAddress) || SMMM.PingType != SBMI.PingHost)
                                        {
                                            foreach (IPAddress Address in SSSHN.GetHostAddresses(Host.Address))
                                            {
                                                try
                                                {
                                                    SBMI.PingAddress = $"{Address}";

                                                    SBMI.NetworkData.PingData = await SSEPPE.SendAsync(SBMI.PingAddress, 1000);

                                                    SBMI.PingHost = SMMM.PingType;
                                                    SBMI.NetworkData.Host = Host.Address;
                                                    SBMI.NetworkData.Ping = SBMI.NetworkData.PingData.RoundTrip;
                                                    SBMI.NetworkData.PingAddress = $"{SBMI.NetworkData.PingData.Address} ({Host.Address})";

                                                    break;
                                                }
                                                catch (Exception Exception)
                                                {
                                                    SBMI.NetworkData.Ping = 0;
                                                    SBMI.PingAddress = string.Empty;
                                                    SBMI.NetworkData.PingData = new();
                                                    SSWW.Watch_CatchException(Exception);
                                                }
                                            }
                                        }
                                        else
                                        {
                                            try
                                            {
                                                SBMI.NetworkData.PingData = await SSEPPE.SendAsync(SBMI.PingAddress, 1000);

                                                SBMI.NetworkData.Host = Host.Address;
                                                SBMI.NetworkData.Ping = SBMI.NetworkData.PingData.RoundTrip;
                                                SBMI.NetworkData.PingAddress = $"{SBMI.NetworkData.PingData.Address} ({Host.Address})";
                                            }
                                            catch (Exception Exception)
                                            {
                                                SBMI.NetworkData.Ping = 0;
                                                SBMI.PingAddress = string.Empty;
                                                SBMI.NetworkData.PingData = new();
                                                SSWW.Watch_CatchException(Exception);
                                            }
                                        }

                                        break;
                                    }
                                }
                            }
                            else
                            {
                                SBMI.NetworkData.Ping = 0;
                                SBMI.NetworkData.PingData = new();
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.PingManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.PingManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.GraphicManagement)
                {
                    SBMI.GraphicManagement = false;

                    _ = Task.Run(() =>
                    {
                        try
                        {
                            string[] Interfaces = SSSHG.AllVideoController();

                            SMMI.SystemSettingManager.SetSetting(SMC.GraphicInterfaces, Interfaces);
                        }
                        catch (Exception Exception)
                        {
                            SBMI.GraphicManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.NetworkManagement)
                {
                    SBMI.NetworkManagement = false;

                    _ = Task.Run(() =>
                    {
                        try
                        {
                            SBMI.NetworkInterfaces = SSSHN.InstanceNetworkInterfaces();

                            SMMI.SystemSettingManager.SetSetting(SMC.NetworkInterfaces, SBMI.NetworkInterfaces);
                        }
                        catch (Exception Exception)
                        {
                            SBMI.NetworkManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.NetworkManagement2 && SBMI.NetworkInterfaces.Any())
                {
                    SBMI.NetworkManagement2 = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SBMI.NetworkInterfaces.Contains(SMMM.NetworkAdapter))
                            {
                                foreach (string Name in SBMI.NetworkInterfaces)
                                {
                                    if (SMMM.NetworkAdapter == Name)
                                    {
                                        if (SMMM.NetworkAdapter != SBMI.NetworkData.Name)
                                        {
                                            SBMI.NetworkData.State = true;
                                            SBMI.NetworkData.Name = SMMM.NetworkAdapter;

                                            SBMI.UploadCounter = new("Network Interface", "Bytes Sent/sec", Name);
                                            SBMI.DownloadCounter = new("Network Interface", "Bytes Received/sec", Name);
                                        }

                                        if (SBMI.UploadCounter != null)
                                        {
                                            SBMI.NetworkData.Upload = SBMI.UploadCounter.NextValue();

                                            SBMI.NetworkData.UploadData = SSESSE.AutoConvert(SBMI.NetworkData.Upload, SEST.Byte, SEMST.Palila);

                                            SBMI.NetworkData.FormatUploadData = SHN.Numeral(SBMI.NetworkData.UploadData.Value, true, true, 2, '0', SECNT.None) + " " + SBMI.NetworkData.UploadData.Text;
                                        }

                                        if (SBMI.DownloadCounter != null)
                                        {
                                            SBMI.NetworkData.Download = SBMI.DownloadCounter.NextValue();

                                            SBMI.NetworkData.DownloadData = SSESSE.AutoConvert(SBMI.NetworkData.Download, SEST.Byte, SEMST.Palila);

                                            SBMI.NetworkData.FormatDownloadData = SHN.Numeral(SBMI.NetworkData.DownloadData.Value, true, true, 2, '0', SECNT.None) + " " + SBMI.NetworkData.DownloadData.Text;
                                        }

                                        break;
                                    }
                                }
                            }
                            else
                            {
                                SBMI.NetworkData.State = false;
                                SBMI.NetworkData.Name = SMMM.NetworkAdapter;
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.NetworkManagement2 = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.NetworkManagement2 = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.MotherboardManagement)
                {
                    SBMI.MotherboardManagement = false;

                    _ = Task.Run(() =>
                    {
                        try
                        {
                            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_BaseBoard");

                            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                            {
                                SBMI.MotherboardData.State = true;
                                SBMI.MotherboardData.Product = SBEO.Check(Object, "Product", string.Empty);
                                SBMI.MotherboardData.Version = SBEO.Check(Object, "Version", string.Empty);
                                SBMI.MotherboardData.Manufacturer = SBEO.Check(Object, "Manufacturer", string.Empty);

                                break;
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.MotherboardManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.FullscreenManagement)
                {
                    SBMI.FullscreenManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            IntPtr Foreground = SWNM.GetForegroundWindow();

                            StringBuilder Class = new(256);
                            SWNM.GetClassName((int)Foreground, Class, 256);

                            if (!SBMI.FocusDesktop)
                            {
                                SWUS.Initialize();

                                foreach (SSMMS Screen in SWUS.Screens)
                                {
                                    if (SWHF.IsFullscreen(Foreground, Screen.rcMonitor))
                                    {
                                        SBMI.FullscreenManagement = true;

                                        SBMI.Fullscreen = true;

                                        return;
                                    }
                                }
                            }

                            SBMI.Fullscreen = false;

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.FullscreenManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.FullscreenManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.VirtualityManagement && (SBMM.VirtualPerformance != SSDEPT.Resume || SBMI.CategoryPerformance == SSDECPT.Virtual))
                {
                    SBMI.VirtualityManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.Virtuality = SBEV.VirtualityActive();

                            await Task.Delay(SBMI.SpecificationTime);

                            SBMI.VirtualityManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.VirtualityManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.FocusManagement)
                {
                    SBMI.FocusManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.FocusDesktop = SWUD.IsDesktopBasic() || SWUD.IsDesktopAdvanced();

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.FocusManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.FocusManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.RemoteManagement && (SBMM.RemotePerformance != SSDEPT.Resume || SBMI.CategoryPerformance == SSDECPT.Remote))
                {
                    SBMI.RemoteManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.RemoteDesktop = SBER.DesktopActive();

                            await Task.Delay(SBMI.SpecificationTime);

                            SBMI.RemoteManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.RemoteManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.ComputerManagement)
                {
                    SBMI.ComputerManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.Computer.Accept(new SBEUV());

                            foreach (IHardware Hardware in SBMI.Computer.Hardware)
                            {
                                if (Hardware.HardwareType == HardwareType.Cpu)
                                {
                                    _ = Task.Run(() =>
                                    {
                                        try
                                        {
                                            //Hardware.Update();

                                            foreach (ISensor Sensor in Hardware.Sensors)
                                            {
                                                if (Sensor.SensorType == SensorType.Load && Sensor.Name == "CPU Total")
                                                {
                                                    SBMI.CpuData.State = true;
                                                    SBMI.CpuData.Min = Sensor.Min;
                                                    SBMI.CpuData.Max = Sensor.Max;
                                                    SBMI.CpuData.Now = Sensor.Value;
                                                    SBMI.CpuData.Name = Hardware.Name;

                                                    break;
                                                }
                                            }
                                        }
                                        catch (Exception Exception)
                                        {
                                            SSWW.Watch_CatchException(Exception);
                                        }
                                    });
                                }
                                else if (Hardware.HardwareType == HardwareType.Memory)
                                {
                                    _ = Task.Run(() =>
                                    {
                                        try
                                        {
                                            //Hardware.Update();

                                            SBMI.MemoryData.State = true;
                                            SBMI.MemoryData.Name = Hardware.Name;

                                            foreach (ISensor Sensor in Hardware.Sensors)
                                            {
                                                switch (Sensor.Name)
                                                {
                                                    case "Memory Used" when Sensor.SensorType == SensorType.Data:
                                                        SBMI.MemoryData.MemoryUsed = Sensor.Value;
                                                        break;
                                                    case "Memory Available" when Sensor.SensorType == SensorType.Data:
                                                        SBMI.MemoryData.MemoryAvailable = Sensor.Value;
                                                        break;
                                                    case "Memory" when Sensor.SensorType == SensorType.Load:
                                                        SBMI.MemoryData.MemoryLoad = Sensor.Value;
                                                        break;
                                                    case "Virtual Memory Used" when Sensor.SensorType == SensorType.Data:
                                                        SBMI.MemoryData.VirtualMemoryUsed = Sensor.Value;
                                                        break;
                                                    case "Virtual Memory Available" when Sensor.SensorType == SensorType.Data:
                                                        SBMI.MemoryData.VirtualMemoryAvailable = Sensor.Value;
                                                        break;
                                                    case "Virtual Memory" when Sensor.SensorType == SensorType.Load:
                                                        SBMI.MemoryData.VirtualMemoryLoad = Sensor.Value;
                                                        break;
                                                    default:
                                                        break;
                                                }
                                            }
                                        }
                                        catch (Exception Exception)
                                        {
                                            SSWW.Watch_CatchException(Exception);
                                        }
                                    });
                                }
                                else if (Hardware.HardwareType == HardwareType.Battery)
                                {
                                    _ = Task.Run(() =>
                                    {
                                        try
                                        {
                                            //Hardware.Update();

                                            if (Hardware.Sensors.Any())
                                            {
                                                SBMI.BatteryData.State = true;
                                                SBMI.BatteryData.Name = Hardware.Name;

                                                foreach (ISensor Sensor in Hardware.Sensors)
                                                {
                                                    switch (Sensor.Name)
                                                    {
                                                        case "Charge Level" when Sensor.SensorType == SensorType.Level:
                                                            SBMI.BatteryData.ChargeLevel = Sensor.Value;
                                                            break;
                                                        case "Discharge Level" when Sensor.SensorType == SensorType.Level:
                                                            SBMI.BatteryData.DischargeLevel = Sensor.Value;
                                                            break;
                                                        case "Voltage" when Sensor.SensorType == SensorType.Voltage:
                                                            SBMI.BatteryData.Voltage = Sensor.Value;
                                                            break;
                                                        case "Charge Current" when Sensor.SensorType == SensorType.Current:
                                                            SBMI.BatteryData.ChargeCurrent = Sensor.Value;
                                                            break;
                                                        case "Discharge Current" when Sensor.SensorType == SensorType.Current:
                                                            SBMI.BatteryData.DischargeCurrent = Sensor.Value;
                                                            break;
                                                        case "Charge / Discharge Current" when Sensor.SensorType == SensorType.Current:
                                                            SBMI.BatteryData.ChargeDischargeCurrent = Sensor.Value;
                                                            break;
                                                        case "Designed Capacity" when Sensor.SensorType == SensorType.Energy:
                                                            SBMI.BatteryData.DesignedCapacity = Sensor.Value;
                                                            break;
                                                        case "Full Charged Capacity" when Sensor.SensorType == SensorType.Energy:
                                                            SBMI.BatteryData.FullChargedCapacity = Sensor.Value;
                                                            break;
                                                        case "Remaining Capacity" when Sensor.SensorType == SensorType.Energy:
                                                            SBMI.BatteryData.RemainingCapacity = Sensor.Value;
                                                            break;
                                                        case "Charge Rate" when Sensor.SensorType == SensorType.Power:
                                                            SBMI.BatteryData.ChargeRate = Sensor.Value;
                                                            break;
                                                        case "Discharge Rate" when Sensor.SensorType == SensorType.Power:
                                                            SBMI.BatteryData.DischargeRate = Sensor.Value;
                                                            break;
                                                        case "Charge / Discharge Rate" when Sensor.SensorType == SensorType.Power:
                                                            SBMI.BatteryData.ChargeDischargeRate = Sensor.Value;
                                                            break;
                                                        case "Degradation Level" when Sensor.SensorType == SensorType.Level:
                                                            SBMI.BatteryData.DegradationLevel = Sensor.Value;
                                                            break;
                                                        case "Remaining Time (Estimated)" when Sensor.SensorType == SensorType.TimeSpan:
                                                            SBMI.BatteryData.RemainingTimeEstimated = Sensor.Value;
                                                            break;
                                                        default:
                                                            break;
                                                    }
                                                }
                                            }
                                            else
                                            {
                                                SBMI.BatteryData.State = false;
                                            }
                                        }
                                        catch (Exception Exception)
                                        {
                                            SSWW.Watch_CatchException(Exception);
                                        }
                                    });
                                }
                                else if (Hardware.HardwareType == HardwareType.Motherboard)
                                {
                                    _ = Task.Run(() =>
                                    {
                                        try
                                        {
                                            //Hardware.Update();

                                            SBMI.MotherboardData.State = true;
                                            SBMI.MotherboardData.Name = Hardware.Name;
                                        }
                                        catch (Exception Exception)
                                        {
                                            SSWW.Watch_CatchException(Exception);
                                        }
                                    });
                                }
                                else if (Hardware.HardwareType is HardwareType.GpuAmd or HardwareType.GpuIntel or HardwareType.GpuNvidia)
                                {
                                    _ = Task.Run(() =>
                                    {
                                        try
                                        {
                                            //Hardware.Update();

                                            List<SBSSSS> Sensors = new()
                                        {
                                            new SBSSSS
                                            {
                                                Name = Hardware.Name,
                                                Type = $"{Hardware.HardwareType}"
                                            }
                                        };

                                            foreach (ISensor Sensor in Hardware.Sensors)
                                            {
                                                Sensors.Add(new SBSSSS
                                                {
                                                    Max = Sensor.Max,
                                                    Min = Sensor.Min,
                                                    Name = Sensor.Name,
                                                    Now = Sensor.Value,
                                                    Type = $"{Sensor.SensorType}"
                                                });
                                            }

                                            string Result = JsonConvert.SerializeObject(Sensors, Formatting.Indented);

                                            switch (Hardware.HardwareType)
                                            {
                                                case HardwareType.GpuAmd:
                                                    SBMI.GraphicData.State = true;
                                                    SBMI.GraphicData.Amd = JArray.Parse(Result);
                                                    break;
                                                case HardwareType.GpuIntel:
                                                    SBMI.GraphicData.State = true;
                                                    SBMI.GraphicData.Intel = JArray.Parse(Result);
                                                    break;
                                                case HardwareType.GpuNvidia:
                                                    SBMI.GraphicData.State = true;
                                                    SBMI.GraphicData.Nvidia = JArray.Parse(Result);
                                                    break;
                                                default:
                                                    break;
                                            }
                                        }
                                        catch (Exception Exception)
                                        {
                                            SSWW.Watch_CatchException(Exception);
                                        }
                                    });
                                }
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.ComputerManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.ComputerManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SMMM.SignalRequired && SBMI.SignalManagement)
                {
                    SBMI.SignalManagement = false;

                    _ = Task.Run(() =>
                    {
                        try
                        {
                            SSMI.BackgroundogManager.FileSave<SSIB>(new()
                            {
                                Cpu = SBED.GetCpuInfo(),
                                Bios = SBED.GetBiosInfo(),
                                Date = SBED.GetDateInfo(),
                                Audio = SBED.GetAudioInfo(),
                                Memory = SBED.GetMemoryInfo(),
                                Battery = SBED.GetBatteryInfo(),
                                Graphic = SBED.GetGraphicInfo(),
                                Network = SBED.GetNetworkInfo(),
                                Motherboard = SBED.GetMotherboardInfo()
                            });

                            SBMI.SignalManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.SignalManagement = true;
                            SSWW.Watch_CatchException(Exception);
                        }
                    });
                }

                //_ = Task.Run(() =>
                //{
                //    foreach (IHardware Hardware in SBMI.Computer.Hardware)
                //    {
                //        Console.WriteLine("Hardware: {0}", Hardware.Name);
                //        Console.WriteLine("Hardware Type: {0}", Hardware.HardwareType);

                //        foreach (IHardware Subhardware in Hardware.SubHardware)
                //        {
                //            Console.WriteLine("\tSubhardware: {0}", Subhardware.Name);

                //            foreach (ISensor Sensor in Subhardware.Sensors)
                //            {
                //                Console.WriteLine("\t\tSensor: {0}, type: {1}, value: {2}", Sensor.Name, Sensor.SensorType, Sensor.Value);
                //            }
                //        }

                //        foreach (ISensor Sensor in Hardware.Sensors)
                //        {
                //            Console.WriteLine("\tSensor: {0}, type: {1}, value: {2}", Sensor.Name, Sensor.SensorType, Sensor.Value);
                //        }
                //    }

                //    Console.WriteLine("----------------------------------------------");
                //});
            }

            await Task.CompletedTask;
        }
    }
}