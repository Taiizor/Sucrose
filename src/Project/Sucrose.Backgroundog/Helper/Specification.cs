using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Diagnostics;
using System.IO;
using System.Management;
using System.Net;
using System.Net.Sockets;
using SBEAS = Sucrose.Backgroundog.Extension.AudioSession;
using SBEC = Sucrose.Backgroundog.Extension.Core;
using SBED = Sucrose.Backgroundog.Extension.Data;
using SBEG = Sucrose.Backgroundog.Extension.Graphic;
using SBER = Sucrose.Backgroundog.Extension.Remote;
using SBES = Sucrose.Backgroundog.Extension.Storage;
using SBEV = Sucrose.Backgroundog.Extension.Virtual;
using SBEVAE = Sucrose.Backgroundog.Enumerators.VorticeAdapterEnumerator;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SBSCS = Sucrose.Backgroundog.Struct.CoreSensor;
using SBSDS = Sucrose.Backgroundog.Struct.DriverSensor;
using SBSSS = Sucrose.Backgroundog.Struct.StorageSensor;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHN = Skylark.Helper.Numeric;
using SHS = Skylark.Helper.Skymath;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SMMCS = Sucrose.Memory.Manage.Constant.System;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SPIB = Sucrose.Pipe.Interface.Backgroundog;
using SPMI = Sucrose.Pipe.Manage.Internal;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSDSH = Sucrose.Shared.Dependency.Struct.Host;
using SSEPPE = Skylark.Standard.Extension.Ping.PingExtension;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSIB = Sucrose.Signal.Interface.Backgroundog;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSMMS = Skylark.Struct.Monitor.MonitorStruct;
using SSPPSS = Skylark.Struct.Ping.PingSendStruct;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSSS = Skylark.Struct.Storage.StorageStruct;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
using STMI = Sucrose.Transmission.Manage.Internal;
using SWHFS = Skylark.Wing.Helper.FullScreen;
using SWNM = Skylark.Wing.Native.Methods;
using SWUD = Skylark.Wing.Utility.Desktop;
using SWUPN = Skylark.Wing.Utility.Plan;
using SWUPR = Skylark.Wing.Utility.Power;
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
                _ = Task.Run(async () =>
                {
                    try
                    {
                        DateTime Date = DateTime.Now;

                        SBMI.DateData = new()
                        {
                            State = true,
                            Day = Date.Day,
                            Hour = Date.Hour,
                            Kind = Date.Kind,
                            Year = Date.Year,
                            Month = Date.Month,
                            Minute = Date.Minute,
                            Second = Date.Second,
                            DayOfWeek = Date.DayOfWeek,
                            DayOfYear = Date.DayOfYear,
                            Millisecond = Date.Millisecond
                        };
                    }
                    catch (Exception Exception)
                    {
                        await SSWEW.Watch_CatchException(Exception);
                    }
                });

                if (SBMI.BiosManagement)
                {
                    SBMI.BiosManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            ManagementClass Class = new("Win32_BIOS");

                            foreach (ManagementObject Object in Class.GetInstances().Cast<ManagementObject>())
                            {
                                SBMI.BiosData.State = true;
                                SBMI.BiosData.Name = SSSHM.Check(Object, "Name", string.Empty);
                                SBMI.BiosData.Caption = SSSHM.Check(Object, "Caption", string.Empty);
                                SBMI.BiosData.Version = SSSHM.Check(Object, "Version", string.Empty);
                                SBMI.BiosData.Description = SSSHM.Check(Object, "Description", string.Empty);
                                SBMI.BiosData.ReleaseDate = SSSHM.Check(Object, "ReleaseDate", string.Empty);
                                SBMI.BiosData.Manufacturer = SSSHM.Check(Object, "Manufacturer", string.Empty);
                                SBMI.BiosData.SerialNumber = SSSHM.Check(Object, "SerialNumber", string.Empty);
                                SBMI.BiosData.CurrentLanguage = SSSHM.Check(Object, "CurrentLanguage", string.Empty);

                                break;
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.BiosManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
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
                            if (await SSSHN.GetHostEntryAsync())
                            {
                                SBMI.NetworkData.Online = true;

                                List<SSDSH> Hosts = SSSHN.GetHost();

                                SSDSH? Host = Hosts.FirstOrDefault(Host => Host.Name == SMMB.PingType);

                                if (Host != null)
                                {
                                    if (string.IsNullOrEmpty(SBMI.PingAddress) || SMMB.PingType != SBMI.PingHost)
                                    {
                                        foreach (IPAddress Address in SSSHN.GetHostAddresses(Host?.Address))
                                        {
                                            try
                                            {
                                                SBMI.PingAddress = $"{Address}";

                                                SSPPSS PingData = await SSEPPE.SendAsync(SBMI.PingAddress, 1000);

                                                SBMI.PingHost = SMMB.PingType;
                                                SBMI.NetworkData.Host = Host?.Address;
                                                SBMI.NetworkData.Ping = PingData.RoundTrip;
                                                SBMI.NetworkData.PingAddress = $"{PingData.Address} ({Host?.Address})";
                                                SBMI.NetworkData.PingData = JObject.Parse(JsonConvert.SerializeObject(PingData, Formatting.Indented));

                                                break;
                                            }
                                            catch (Exception Exception)
                                            {
                                                SBMI.NetworkData.Ping = 0;
                                                SBMI.PingAddress = string.Empty;
                                                SBMI.NetworkData.PingData = [];
                                                await SSWEW.Watch_CatchException(Exception);
                                            }
                                        }
                                    }
                                    else
                                    {
                                        try
                                        {
                                            SSPPSS PingData = await SSEPPE.SendAsync(SBMI.PingAddress, 1000);

                                            SBMI.NetworkData.Host = Host?.Address;
                                            SBMI.NetworkData.Ping = PingData.RoundTrip;
                                            SBMI.NetworkData.PingAddress = $"{PingData.Address} ({Host?.Address})";
                                            SBMI.NetworkData.PingData = JObject.Parse(JsonConvert.SerializeObject(PingData, Formatting.Indented));
                                        }
                                        catch (Exception Exception)
                                        {
                                            SBMI.NetworkData.Ping = 0;
                                            SBMI.PingAddress = string.Empty;
                                            SBMI.NetworkData.PingData = [];
                                            await SSWEW.Watch_CatchException(Exception);
                                        }
                                    }
                                }
                            }
                            else
                            {
                                SBMI.NetworkData.Ping = 0;
                                SBMI.NetworkData.Online = false;
                                SBMI.NetworkData.PingData = [];
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.PingManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.PingManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.AudioManagement)
                {
                    SBMI.AudioManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SMMB.AudioRequired)
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
                                            SBMI.AudioData.Data = e.Mono;
                                            SBMI.AudioData.DataLeft = e.Left;
                                            SBMI.AudioData.DataRight = e.Right;
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
                            else
                            {

                                SBMI.DataSource = null;
                                SBMI.SessionManager = null;
                                SBMI.AudioManagement = true;
                                SBMI.AudioData.State = false;
                                SBMI.SessionManagement = true;

                                try
                                {
                                    SBMI.AudioVisualizer?.Stop();
                                    SBMI.SessionManager?.SessionListChanged -= (s, e) => SBEAS.SessionListChanged();
                                }
                                catch { }
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.AudioManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.MemoryManagement)
                {
                    SBMI.MemoryManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SWNM.MEMORYSTATUSEX MemoryStatus = new();

                            if (SWNM.GlobalMemoryStatusEx(MemoryStatus))
                            {
                                SBMI.MemoryData.State = true;

                                SBMI.MemoryData.Name = "Total Memory";
                                SBMI.MemoryData.MemoryAvailable = (float)MemoryStatus.ullAvailPhys / (1024 * 1024 * 1024);
                                SBMI.MemoryData.MemoryLoad = 100.0f - (100.0f * MemoryStatus.ullAvailPhys / MemoryStatus.ullTotalPhys);
                                SBMI.MemoryData.MemoryUsed = (float)(MemoryStatus.ullTotalPhys - MemoryStatus.ullAvailPhys) / (1024 * 1024 * 1024);

                                SBMI.MemoryData.VirtualName = "Virtual Memory";
                                SBMI.MemoryData.VirtualMemoryAvailable = (float)MemoryStatus.ullAvailPageFile / (1024 * 1024 * 1024);
                                SBMI.MemoryData.VirtualMemoryLoad = 100.0f - (100.0f * MemoryStatus.ullAvailPageFile / MemoryStatus.ullTotalPageFile);
                                SBMI.MemoryData.VirtualMemoryUsed = (float)(MemoryStatus.ullTotalPageFile - MemoryStatus.ullAvailPageFile) / (1024 * 1024 * 1024);
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.MemoryManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.MemoryManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
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
                            ManagementClass Class = new("Win32_Battery");

                            foreach (ManagementObject Object in Class.GetInstances().Cast<ManagementObject>())
                            {
                                SBMI.BatteryData.State = true;

                                SBMI.BatteryData.Chemistry = SSSHM.Check(Object, "Chemistry", 0);
                                SBMI.BatteryData.Name = SSSHM.Check(Object, "Name", string.Empty);
                                SBMI.BatteryData.Status = SSSHM.Check(Object, "Status", string.Empty);
                                SBMI.BatteryData.DesignVoltage = SSSHM.Check(Object, "DesignVoltage", 0);
                                SBMI.BatteryData.EstimatedRunTime = SSSHM.Check(Object, "EstimatedRunTime", 0);
                                SBMI.BatteryData.Description = SSSHM.Check(Object, "Description", string.Empty);
                                SBMI.BatteryData.EstimatedChargeRemaining = SSSHM.Check(Object, "EstimatedChargeRemaining", 0);

                                break;
                            }

                            SBMI.BatteryData.SavingMode = SWUPR.IsBatterySavingMode;
                            SBMI.BatteryData.BatteryFlag = $"{SWUPR.GetBatteryFlag()}";
                            SBMI.BatteryData.BatteryLifeTime = SWUPR.GetBatteryLifeTime();
                            SBMI.BatteryData.ACPowerStatus = $"{SWUPR.GetACPowerStatus()}";
                            SBMI.BatteryData.EnergySaverType = SWUPR.GetEnergySaverState();
                            SBMI.BatteryData.SaverStatus = $"{SWUPR.GetBatterySaverStatus()}";
                            SBMI.BatteryData.BatteryLifePercent = SWUPR.GetBatteryLifePercent();
                            SBMI.BatteryData.BatteryFullLifeTime = SWUPR.GetBatteryFullLifeTime();

                            SBMI.BatteryData.LifePercent = SystemInformation.PowerStatus.BatteryLifePercent;
                            SBMI.BatteryData.PowerLineStatus = SystemInformation.PowerStatus.PowerLineStatus;
                            SBMI.BatteryData.FullLifetime = SystemInformation.PowerStatus.BatteryFullLifetime;
                            SBMI.BatteryData.ChargeStatus = SystemInformation.PowerStatus.BatteryChargeStatus;
                            SBMI.BatteryData.LifeRemaining = SystemInformation.PowerStatus.BatteryLifeRemaining;

                            SBMI.BatteryData.PowerPlanType = SWUPN.GetPlanFromGuid(SWUPN.GetActivePowerSchemeGuid());

                            await Task.Delay(SBMI.SpecificationTime);

                            SBMI.BatteryManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.BatteryManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.GraphicManagement)
                {
                    SBMI.GraphicManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.GraphicInterfaces = SBEVAE.EnumerateAdaptersDescription();

                            SMMI.SystemSettingManager.SetSetting(SMMCS.GraphicInterfaces, SBMI.GraphicInterfaces);

                            if (SBMI.GraphicInterfaces.Any() && (string.IsNullOrEmpty(SMMB.GraphicAdapter) || !SBMI.GraphicInterfaces.Contains(SMMB.GraphicAdapter)))
                            {
                                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.GraphicAdapter, SBMI.GraphicInterfaces.FirstOrDefault());
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.GraphicManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.NetworkManagement)
                {
                    SBMI.NetworkManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.NetworkInterfaces = SSSHN.InstanceNetworkInterfaces();

                            SMMI.SystemSettingManager.SetSetting(SMMCS.NetworkInterfaces, SBMI.NetworkInterfaces);

                            if (SBMI.NetworkInterfaces.Any() && (string.IsNullOrEmpty(SMMB.NetworkAdapter) || !SBMI.NetworkInterfaces.Contains(SMMB.NetworkAdapter)))
                            {
                                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.NetworkAdapter, SBMI.NetworkInterfaces.FirstOrDefault());
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.NetworkManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.StorageManagement)
                {
                    SBMI.StorageManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            List<SBSSS> Sensors = [];

                            ManagementClass Class = new("Win32_LogicalDisk");

                            foreach (ManagementObject Object in Class.GetInstances().Cast<ManagementObject>())
                            {
                                SBMI.StorageData.State = true;

                                SBSSS Sensor = new()
                                {
                                    Size = SSSHM.Check(Object, "Size", 0L),
                                    DriveType = SSSHM.Check(Object, "DriveType", 0),
                                    MediaType = SSSHM.Check(Object, "MediaType", 0),
                                    FreeSpace = SSSHM.Check(Object, "FreeSpace", 0L),
                                    Name = SSSHM.Check(Object, "Name", string.Empty),
                                    Caption = SSSHM.Check(Object, "Caption", string.Empty),
                                    FileSystem = SSSHM.Check(Object, "FileSystem", string.Empty),
                                    VolumeName = SSSHM.Check(Object, "VolumeName", string.Empty),
                                    Description = SSSHM.Check(Object, "Description", string.Empty),
                                    MaximumComponentLength = SSSHM.Check(Object, "MaximumComponentLength", 0),
                                    Compressed = Convert.ToBoolean(SSSHM.Check(Object, "Compressed", "False")),
                                    VolumeSerialNumber = SSSHM.Check(Object, "VolumeSerialNumber", string.Empty),
                                    SupportsDiskQuotas = Convert.ToBoolean(SSSHM.Check(Object, "SupportsDiskQuotas", "False")),
                                    SupportsFileBasedCompression = Convert.ToBoolean(SSSHM.Check(Object, "SupportsFileBasedCompression", "False"))
                                };

                                Sensor.SizeData = SSESSE.AutoConvert(Sensor.Size ?? 0, SEST.Byte, SEMST.Palila);
                                Sensor.FormatSizeData = SHN.Numeral(Sensor.SizeData.Value, true, true, 2, '0', SECNT.None) + " " + Sensor.SizeData.TypeText;

                                Sensor.FreeSpaceData = SSESSE.AutoConvert(Sensor.FreeSpace ?? 0, SEST.Byte, SEMST.Palila);
                                Sensor.FormatFreeSpaceData = SHN.Numeral(Sensor.FreeSpaceData.Value, true, true, 2, '0', SECNT.None) + " " + Sensor.FreeSpaceData.TypeText;

                                Sensor.UsedSpace = Sensor.Size - Sensor.FreeSpace;
                                Sensor.UsedSpaceData = SSESSE.AutoConvert(Sensor.UsedSpace ?? 0, SEST.Byte, SEMST.Palila);
                                Sensor.FormatUsedSpaceData = SHN.Numeral(Sensor.UsedSpaceData.Value, true, true, 2, '0', SECNT.None) + " " + Sensor.UsedSpaceData.TypeText;

                                Sensors.Add(Sensor);
                            }

                            string Result = JsonConvert.SerializeObject(Sensors, Formatting.Indented);

                            SBMI.StorageData.Drivers = JArray.Parse(Result);

                            await Task.Delay(SBMI.SpecificationPeakTime);

                            SBMI.StorageManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.StorageManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.GraphicManagement2)
                {
                    SBMI.GraphicManagement2 = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SBMI.GraphicInterfaces.Any())
                            {
                                if (string.IsNullOrWhiteSpace(SBMI.GraphicData.Name) || SBMI.GraphicData.Name != SMMB.GraphicAdapter)
                                {
                                    if (SBMI.GraphicCounter != null)
                                    {
                                        SBMI.GraphicCounter.ForEach(Counter =>
                                        {
                                            Counter?.Dispose();
                                        });

                                        SBMI.GraphicCounter = null;
                                    }

                                    SBEVAE.GetNameToLuidMapping().TryGetValue(SMMB.GraphicAdapter, out SBMI.GraphicData.Luid);

                                    SBMI.GraphicCounter = SBEG.CreateCounters("GPU Engine", SBMI.GraphicData.Luid ?? SMMRG.Unknown);

                                    SBEG.InstanceValues(SBMI.GraphicCounter);

                                    SBMI.GraphicData.Max = 0f;
                                    SBMI.GraphicData.Now = 0f;
                                    SBMI.GraphicData.Min = 100f;
                                    SBMI.GraphicData.State = true;
                                    SBMI.GraphicData.Name = SMMB.GraphicAdapter;
                                    SBMI.GraphicData.Instances = SBEG.GetInstances("GPU Engine");
                                    SBMI.GraphicData.Manufacturer = SBEVAE.GetGpuVendorNameByName(SMMB.GraphicAdapter);
                                }
                                else
                                {
                                    SBMI.GraphicData.Now = SHS.Clamp(SBEG.GetValue(SBMI.GraphicCounter), 0f, 100f);

                                    SBEG.UpdateCounters(SBMI.GraphicCounter, ref SBMI.GraphicData.Instances, "GPU Engine", SBMI.GraphicData.Luid ?? SMMRG.Unknown);

                                    SBMI.GraphicData.Max = SHS.Clamp((SBMI.GraphicData.Now > SBMI.GraphicData.Max ? SBMI.GraphicData.Now : SBMI.GraphicData.Max) ?? 0f, 0f, 100f);
                                    SBMI.GraphicData.Min = SHS.Clamp((SBMI.GraphicData.Now < SBMI.GraphicData.Min ? SBMI.GraphicData.Now : SBMI.GraphicData.Min) ?? 100f, 0f, 100f);
                                }
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.GraphicManagement2 = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.GraphicManagement2 = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.StorageManagement2)
                {
                    SBMI.StorageManagement2 = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SBMI.LogicalsCounter == null)
                            {
                                SBMI.LogicalsCounter = SBES.CreateCounters("LogicalDisk");

                                SBES.InstanceValues(SBMI.LogicalsCounter);
                            }
                            else
                            {
                                List<SBSDS> Sensors = [];

                                foreach ((string Instance, float Read, float Write) in SBES.GetValues(SBMI.LogicalsCounter))
                                {
                                    SSSSS ReadData = SSESSE.AutoConvert(Read, SEST.Byte, SEMST.Palila);
                                    SSSSS WriteData = SSESSE.AutoConvert(Write, SEST.Byte, SEMST.Palila);

                                    Sensors.Add(new SBSDS
                                    {
                                        Read = Read,
                                        Write = Write,
                                        Name = Instance,
                                        ReadData = ReadData,
                                        WriteData = WriteData,
                                        FormatReadData = SHN.Numeral(ReadData.Value, true, true, 2, '0', SECNT.None) + " " + ReadData.TypeText,
                                        FormatWriteData = SHN.Numeral(WriteData.Value, true, true, 2, '0', SECNT.None) + " " + WriteData.TypeText,
                                    });
                                }

                                string Result = JsonConvert.SerializeObject(Sensors, Formatting.Indented);

                                SBMI.StorageData.LogicalDrivers = JArray.Parse(Result);
                            }


                            if (SBMI.PhysicalsCounter == null)
                            {
                                SBMI.PhysicalsCounter = SBES.CreateCounters("PhysicalDisk");

                                SBES.InstanceValues(SBMI.PhysicalsCounter);
                            }
                            else
                            {
                                List<SBSDS> Sensors = [];

                                foreach ((string Instance, float Read, float Write) in SBES.GetValues(SBMI.PhysicalsCounter))
                                {
                                    SSSSS ReadData = SSESSE.AutoConvert(Read, SEST.Byte, SEMST.Palila);
                                    SSSSS WriteData = SSESSE.AutoConvert(Write, SEST.Byte, SEMST.Palila);

                                    Sensors.Add(new SBSDS
                                    {
                                        Read = Read,
                                        Write = Write,
                                        Name = Instance,
                                        ReadData = ReadData,
                                        WriteData = WriteData,
                                        FormatReadData = SHN.Numeral(ReadData.Value, true, true, 2, '0', SECNT.None) + " " + ReadData.TypeText,
                                        FormatWriteData = SHN.Numeral(WriteData.Value, true, true, 2, '0', SECNT.None) + " " + WriteData.TypeText,
                                    });
                                }

                                string Result = JsonConvert.SerializeObject(Sensors, Formatting.Indented);

                                SBMI.StorageData.PhysicalDrivers = JArray.Parse(Result);
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.StorageManagement2 = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.StorageManagement2 = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.ProcessorManagement)
                {
                    SBMI.ProcessorManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            ManagementClass Class = new("Win32_Processor");

                            foreach (ManagementObject Object in Class.GetInstances().Cast<ManagementObject>())
                            {
                                SBMI.ProcessorData.State = true;
                                SBMI.ProcessorData.ProcessorCount = Environment.ProcessorCount;
                                SBMI.ProcessorData.Core = SSSHM.Check(Object, "NumberOfCores", 0);
                                SBMI.ProcessorData.Name = SSSHM.Check(Object, "Name", string.Empty);
                                SBMI.ProcessorData.Thread = SSSHM.Check(Object, "NumberOfLogicalProcessors", 0);

                                break;
                            }

                            SMMI.SystemSettingManager.SetSetting(SMMCS.ProcessorInterfaces, SSSHU.GetProcessor());
                        }
                        catch (Exception Exception)
                        {
                            SBMI.ProcessorManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.ProcessorManagement2)
                {
                    SBMI.ProcessorManagement2 = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            if (SBMI.ProcessorCounter == null)
                            {
                                SBMI.ProcessorCounter = new("Processor", "% Processor Time", "_Total");

                                _ = SBMI.ProcessorCounter.NextValue();
                            }
                            else
                            {
                                SBMI.ProcessorData.Type = $"{SBMI.ProcessorCounter.CounterType}";

                                SBMI.ProcessorData.Now = SHS.Clamp(SBMI.ProcessorCounter.NextValue(), 0f, 100f);

                                SBMI.ProcessorData.Max = SHS.Clamp((SBMI.ProcessorData.Now > SBMI.ProcessorData.Max ? SBMI.ProcessorData.Now : SBMI.ProcessorData.Max) ?? 0f, 0f, 100f);
                                SBMI.ProcessorData.Min = SHS.Clamp((SBMI.ProcessorData.Now < SBMI.ProcessorData.Min ? SBMI.ProcessorData.Now : SBMI.ProcessorData.Min) ?? 100f, 0f, 100f);
                            }

                            if (SBMI.ProcessorsCounter == null)
                            {
                                SBMI.ProcessorsCounter = new PerformanceCounter[Environment.ProcessorCount];

                                for (int Core = 0; Core < SBMI.ProcessorsCounter.Length; Core++)
                                {
                                    SBMI.ProcessorsCounter[Core] = new PerformanceCounter("Processor", "% Processor Time", $"{Core}");

                                    _ = SBMI.ProcessorsCounter[Core].NextValue();
                                }
                            }
                            else
                            {
                                List<SBSCS> Sensors = [];

                                SBMI.ProcessorData.CoresNow = 0f;

                                for (int Core = 0; Core < SBMI.ProcessorsCounter.Length; Core++)
                                {
                                    float Now = SHS.Clamp(SBMI.ProcessorsCounter[Core].NextValue(), 0f, 100f);

                                    SBMI.ProcessorData.CoresNow = SHS.Clamp((SBMI.ProcessorData.CoresNow > Now ? SBMI.ProcessorData.CoresNow : Now) ?? 0f, 0f, 100f);

                                    SBMI.ProcessorData.CoresMax = SHS.Clamp((SBMI.ProcessorData.CoresNow > SBMI.ProcessorData.CoresMax ? SBMI.ProcessorData.CoresNow : SBMI.ProcessorData.CoresMax) ?? 0f, 0f, 100f);
                                    SBMI.ProcessorData.CoresMin = SHS.Clamp((SBMI.ProcessorData.CoresNow < SBMI.ProcessorData.CoresMin ? SBMI.ProcessorData.CoresNow : SBMI.ProcessorData.CoresMin) ?? 100f, 0f, 100f);

                                    Sensors.Add(new SBSCS
                                    {
                                        Now = Now,
                                        Index = Core,
                                        IsMax = false,
                                        IsMin = false,
                                        Name = $"Core #{Core}",
                                        Type = SBMI.ProcessorsCounter[Core].CounterType,
                                        TypeText = $"{SBMI.ProcessorsCounter[Core].CounterType}"
                                    });
                                }

                                Sensors = SBEC.UpdateExtremes(Sensors);

                                string Result = JsonConvert.SerializeObject(Sensors, Formatting.Indented);

                                SBMI.ProcessorData.Cores = JArray.Parse(Result);
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.ProcessorManagement2 = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.ProcessorManagement2 = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.MotherboardManagement)
                {
                    SBMI.MotherboardManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            ManagementClass Class = new("Win32_BaseBoard");

                            foreach (ManagementObject Object in Class.GetInstances().Cast<ManagementObject>())
                            {
                                SBMI.MotherboardData.State = true;
                                SBMI.MotherboardData.Product = SSSHM.Check(Object, "Product", string.Empty);
                                SBMI.MotherboardData.Version = SSSHM.Check(Object, "Version", string.Empty);
                                SBMI.MotherboardData.Manufacturer = SSSHM.Check(Object, "Manufacturer", string.Empty);
                                SBMI.MotherboardData.Name = $"{SBMI.MotherboardData.Manufacturer} {SBMI.MotherboardData.Product}";

                                break;
                            }
                        }
                        catch (Exception Exception)
                        {
                            SBMI.MotherboardManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
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
                            if (SBMI.NetworkInterfaces.Contains(SMMB.NetworkAdapter))
                            {
                                foreach (string Name in SBMI.NetworkInterfaces)
                                {
                                    if (SMMB.NetworkAdapter == Name)
                                    {
                                        if (SMMB.NetworkAdapter != SBMI.NetworkData.Name)
                                        {
                                            SBMI.NetworkData.State = true;
                                            SBMI.NetworkData.Name = SMMB.NetworkAdapter;

                                            SBMI.UploadCounter = new("Network Interface", "Bytes Sent/sec", Name);
                                            SBMI.DownloadCounter = new("Network Interface", "Bytes Received/sec", Name);
                                        }

                                        if (SBMI.UploadCounter != null)
                                        {
                                            SBMI.NetworkData.Upload = SBMI.UploadCounter.NextValue();

                                            SSSSS UploadData = SSESSE.AutoConvert(SBMI.NetworkData.Upload, SEST.Byte, SEMST.Palila);

                                            SBMI.NetworkData.UploadData = JObject.Parse(JsonConvert.SerializeObject(UploadData, Formatting.Indented));
                                            SBMI.NetworkData.FormatUploadData = SHN.Numeral(UploadData.Value, true, true, 2, '0', SECNT.None) + " " + UploadData.TypeText;
                                        }

                                        if (SBMI.DownloadCounter != null)
                                        {
                                            SBMI.NetworkData.Download = SBMI.DownloadCounter.NextValue();

                                            SSSSS DownloadData = SSESSE.AutoConvert(SBMI.NetworkData.Download, SEST.Byte, SEMST.Palila);

                                            SBMI.NetworkData.DownloadData = JObject.Parse(JsonConvert.SerializeObject(DownloadData, Formatting.Indented));
                                            SBMI.NetworkData.FormatDownloadData = SHN.Numeral(DownloadData.Value, true, true, 2, '0', SECNT.None) + " " + DownloadData.TypeText;
                                        }

                                        SBMI.NetworkData.Total = SBMI.NetworkData.Upload + SBMI.NetworkData.Download;

                                        SSSSS TotalData = SSESSE.AutoConvert(SBMI.NetworkData.Total, SEST.Byte, SEMST.Palila);

                                        SBMI.NetworkData.TotalData = JObject.Parse(JsonConvert.SerializeObject(TotalData, Formatting.Indented));
                                        SBMI.NetworkData.FormatTotalData = SHN.Numeral(TotalData.Value, true, true, 2, '0', SECNT.None) + " " + TotalData.TypeText;

                                        break;
                                    }
                                }
                            }
                            else
                            {
                                SBMI.NetworkData.State = false;
                                SBMI.NetworkData.Name = SMMB.NetworkAdapter;
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.NetworkManagement2 = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.NetworkManagement2 = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.RemoteManagement && (SSDMMB.RemotePerformance != SSDEPT.Resume || SBMI.CategoryPerformance == SSDECPT.Remote))
                {
                    SBMI.RemoteManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.RemoteDesktop = SBMI.WindowsRemote || SBER.RemotelyActive();

                            await Task.Delay(SBMI.SpecificationTime);

                            SBMI.RemoteManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.RemoteManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.VirtualityManagement && (SSDMMB.VirtualPerformance != SSDEPT.Resume || SBMI.CategoryPerformance == SSDECPT.Virtual))
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
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.FullScreenManagement && (SSDMMB.FullScreenPerformance != SSDEPT.Resume || SBMI.CategoryPerformance == SSDECPT.FullScreen))
                {
                    SBMI.FullScreenManagement = false;

                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            SBMI.FullScreen = false;

                            if (!SBMI.FocusDesktop)
                            {
                                SWUS.Initialize();

                                IntPtr Foreground = SWNM.GetForegroundWindow();

                                foreach (SSMMS Screen in SWUS.Screens)
                                {
                                    if (SWHFS.IsFullScreen(Foreground, Screen.rcMonitor))
                                    {
                                        SBMI.FullScreen = true;

                                        break;
                                    }
                                }
                            }

                            await Task.Delay(SBMI.SpecificationLessTime);

                            SBMI.FullScreenManagement = true;
                        }
                        catch (Exception Exception)
                        {
                            SBMI.FullScreenManagement = true;
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                if (SBMI.FocusManagement && (SSDMMB.FocusPerformance != SSDEPT.Resume || SSDMMB.FullScreenPerformance != SSDEPT.Resume || SBMI.CategoryPerformance == SSDECPT.Focus || SBMI.CategoryPerformance == SSDECPT.FullScreen))
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
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });
                }

                _ = Task.Run(async () =>
                {
                    try
                    {
                        if (!SBMI.Condition && SBMI.PipeManagement && SMMB.PipeRequired)
                        {
                            SBMI.PipeManagement = false;

                            JsonSerializerSettings SerializerSettings = new()
                            {
                                Formatting = Formatting.None,
                                TypeNameHandling = TypeNameHandling.None
                            };

                            try
                            {
                                await SPMI.BackgroundogManager.StartClient(JsonConvert.SerializeObject(new SPIB()
                                {
                                    Bios = SBED.GetBiosInfo(),
                                    Date = SBED.GetDateInfo(),
                                    Audio = SBED.GetAudioInfo(),
                                    Memory = SBED.GetMemoryInfo(),
                                    Battery = SBED.GetBatteryInfo(),
                                    Graphic = SBED.GetGraphicInfo(),
                                    Network = SBED.GetNetworkInfo(),
                                    Storage = SBED.GetStorageInfo(),
                                    Processor = SBED.GetProcessorInfo(),
                                    Motherboard = SBED.GetMotherboardInfo()
                                }, SerializerSettings));
                            }
                            catch (InvalidOperationException Exception) when (Exception.Message.Contains("Failed to connect to pipe after"))
                            {
                                // Clean up the failed manager
                                try
                                {
                                    await SPMI.BackgroundogManager.DisposeClient();
                                }
                                catch { }

                                // Connection retries failed, log and skip this cycle
                                await SSWEW.Watch_CatchException(Exception);
                            }
                            catch (TimeoutException Exception)
                            {
                                // Clean up the failed manager
                                try
                                {
                                    await SPMI.BackgroundogManager.DisposeClient();
                                }
                                catch { }

                                // Connection timeout, log and skip this cycle
                                await SSWEW.Watch_CatchException(Exception);
                            }

                            SBMI.PipeManagement = true;
                        }
                    }
                    catch (IOException Exception)
                    {
                        // Clean up the existing manager before creating a new one
                        if (SPMI.BackgroundogManager != null)
                        {
                            try
                            {
                                await SPMI.BackgroundogManager.DisposeClient();
                            }
                            catch { }
                        }

                        SBMI.PipeManagement = true;
                        await SSWEW.Watch_CatchException(Exception);
                    }
                    catch (Exception Exception)
                    {
                        SBMI.PipeManagement = true;
                        await SSWEW.Watch_CatchException(Exception);
                    }
                });

                _ = Task.Run(async () =>
                {
                    try
                    {
                        if (!SBMI.Condition && SBMI.SignalManagement && SMMB.SignalRequired)
                        {
                            SBMI.SignalManagement = false;

                            SSMI.BackgroundogManager.FileSave<SSIB>(new()
                            {
                                Bios = SBED.GetBiosInfo(),
                                Date = SBED.GetDateInfo(),
                                Audio = SBED.GetAudioInfo(),
                                Memory = SBED.GetMemoryInfo(),
                                Battery = SBED.GetBatteryInfo(),
                                Graphic = SBED.GetGraphicInfo(),
                                Network = SBED.GetNetworkInfo(),
                                Storage = SBED.GetStorageInfo(),
                                Processor = SBED.GetProcessorInfo(),
                                Motherboard = SBED.GetMotherboardInfo()
                            });
                        }
                    }
                    catch (InvalidOperationException Exception) when (Exception.Message.Contains("Failed to write file after"))
                    {
                        // Signal write failed after retries
                        await SSWEW.Watch_CatchException(Exception);
                    }
                    catch (Exception Exception)
                    {
                        await SSWEW.Watch_CatchException(Exception);
                    }

                    SBMI.SignalManagement = true;
                });

                _ = Task.Run(async () =>
                {
                    try
                    {
                        if (!SBMI.Condition && SBMI.TransmissionManagement && SMMB.TransmissionRequired)
                        {
                            SBMI.TransmissionManagement = false;

                            STMI.BackgroundogManager ??= new(SMMRG.Loopback, SMMB.TransmissionPort);

                            JsonSerializerSettings SerializerSettings = new()
                            {
                                Formatting = Formatting.None,
                                TypeNameHandling = TypeNameHandling.None
                            };

                            try
                            {
                                await STMI.BackgroundogManager.StartClient(JsonConvert.SerializeObject(new SPIB()
                                {
                                    Bios = SBED.GetBiosInfo(),
                                    Date = SBED.GetDateInfo(),
                                    Audio = SBED.GetAudioInfo(),
                                    Memory = SBED.GetMemoryInfo(),
                                    Battery = SBED.GetBatteryInfo(),
                                    Graphic = SBED.GetGraphicInfo(),
                                    Network = SBED.GetNetworkInfo(),
                                    Storage = SBED.GetStorageInfo(),
                                    Processor = SBED.GetProcessorInfo(),
                                    Motherboard = SBED.GetMotherboardInfo()
                                }, SerializerSettings));
                            }
                            catch (InvalidOperationException Exception) when (Exception.Message.Contains("Failed to connect after"))
                            {
                                // Clean up the failed manager
                                try
                                {
                                    await STMI.BackgroundogManager.DisposeClient();
                                }
                                catch { }

                                STMI.BackgroundogManager = null;

                                // Connection retries failed, log and skip this cycle
                                await SSWEW.Watch_CatchException(Exception);
                            }
                            catch (TimeoutException Exception)
                            {
                                // Clean up the failed manager
                                try
                                {
                                    await STMI.BackgroundogManager.DisposeClient();
                                }
                                catch { }

                                STMI.BackgroundogManager = null;

                                // Connection timeout, log and skip this cycle
                                await SSWEW.Watch_CatchException(Exception);
                            }

                            SBMI.TransmissionManagement = true;
                        }
                    }
                    catch (SocketException Exception)
                    {
                        // Clean up the existing manager before creating a new one
                        if (STMI.BackgroundogManager != null)
                        {
                            try
                            {
                                await STMI.BackgroundogManager.DisposeClient();
                            }
                            catch { }
                        }

                        STMI.BackgroundogManager = null;
                        SBMI.TransmissionManagement = true;

                        await SSWEW.Watch_CatchException(Exception);
                    }
                    catch (Exception Exception)
                    {
                        SBMI.TransmissionManagement = true;
                        await SSWEW.Watch_CatchException(Exception);
                    }
                });
            }

            await Task.CompletedTask;
        }
    }
}