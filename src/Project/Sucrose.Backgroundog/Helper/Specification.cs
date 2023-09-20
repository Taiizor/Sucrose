using LibreHardwareMonitor.Hardware;
using Skylark.Enum;
using Skylark.Helper;
using Skylark.Standard.Extension.Storage;
using System.Management;
using SBEAS = Sucrose.Backgroundog.Extension.AudioSession;
using SBEUV = Sucrose.Backgroundog.Extension.UpdateVisitor;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSSHN = Sucrose.Shared.Space.Helper.Network;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Specification
    {
        public static async Task Start()
        {
            if (SBMI.Exit)
            {
                _ = Task.Run(() =>
                {
                    if (SBMI.CpuManagement)
                    {
                        SBMI.CpuManagement = false;

                        ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

                        foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                        {
                            SBMI.CpuData.State = true;
                            SBMI.CpuData.Core = Convert.ToInt32(Object["NumberOfCores"]);
                            SBMI.CpuData.Fullname = Object["Name"].ToString().TrimStart().TrimEnd();
                            SBMI.CpuData.Thread = Convert.ToInt32(Object["NumberOfLogicalProcessors"]);
                            break;
                        }
                    }
                });

                _ = Task.Run(() =>
                {
                    if (SBMI.BiosManagement)
                    {
                        SBMI.BiosManagement = false;

                        ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_BIOS");

                        foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                        {
                            SBMI.BiosData.State = true;
                            SBMI.BiosData.Name = Object["Name"].ToString();
                            SBMI.BiosData.Caption = Object["Caption"].ToString();
                            SBMI.BiosData.Version = Object["Version"].ToString();
                            SBMI.BiosData.Description = Object["Description"].ToString();
                            SBMI.BiosData.ReleaseDate = Object["ReleaseDate"].ToString();
                            SBMI.BiosData.Manufacturer = Object["Manufacturer"].ToString();
                            SBMI.BiosData.SerialNumber = Object["SerialNumber"].ToString();
                            SBMI.BiosData.CurrentLanguage = Object["CurrentLanguage"].ToString();

                            break;
                        }
                    }
                });

                _ = Task.Run(() =>
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
                });

                _ = Task.Run(() =>
                {
                    if (SBMI.AudioManagement)
                    {
                        SBMI.AudioManagement = false;

                        SBMI.SessionManager.SessionListChanged += (s, e) => SBEAS.SessionListChanged();
                    }

                    SBEAS.SessionListChanged();
                });

                _ = Task.Run(() =>
                {
                    string[] Interfaces = SSSHN.InstanceNetworkInterfaces();

                    SMMI.SystemSettingManager.SetSetting(SMC.NetworkInterfaces, Interfaces);

                    if (Interfaces.Contains(SMMM.NetworkAdapter))
                    {
                        foreach (string Name in Interfaces)
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

                                SBMI.NetworkData.Upload = SBMI.UploadCounter.NextValue();
                                SBMI.NetworkData.Download = SBMI.DownloadCounter.NextValue();

                                SBMI.NetworkData.UploadData = StorageExtension.AutoConvert(SBMI.NetworkData.Upload, StorageType.Byte, ModeStorageType.Palila);
                                SBMI.NetworkData.DownloadData = StorageExtension.AutoConvert(SBMI.NetworkData.Download, StorageType.Byte, ModeStorageType.Palila);

                                SBMI.NetworkData.FormatUploadData = Numeric.Numeral(SBMI.NetworkData.UploadData.Value, true, true, 2, '0', ClearNumericType.None) + " " + SBMI.NetworkData.UploadData.Text;
                                SBMI.NetworkData.FormatDownloadData = Numeric.Numeral(SBMI.NetworkData.DownloadData.Value, true, true, 2, '0', ClearNumericType.None) + " " + SBMI.NetworkData.DownloadData.Text;

                                break;
                            }
                        }
                    }
                    else
                    {
                        SBMI.NetworkData.State = false;
                        SBMI.NetworkData.Name = SMMM.NetworkAdapter;
                    }
                });

                _ = Task.Run(() =>
                {
                    if (SBMI.MotherboardManagement)
                    {
                        SBMI.MotherboardManagement = false;

                        ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_BaseBoard");

                        foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                        {
                            SBMI.MotherboardData.State = true;
                            SBMI.MotherboardData.Product = Object["Product"].ToString();
                            SBMI.MotherboardData.Version = Object["Version"].ToString();
                            SBMI.MotherboardData.Manufacturer = Object["Manufacturer"].ToString();

                            break;
                        }
                    }
                });

                _ = Task.Run(() =>
                {
                    SBMI.Computer.Accept(new SBEUV());

                    foreach (IHardware Hardware in SBMI.Computer.Hardware)
                    {
                        if (Hardware.HardwareType == HardwareType.Cpu)
                        {
                            Hardware.Update();

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
                        else if (Hardware.HardwareType == HardwareType.Memory)
                        {
                            Hardware.Update();

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
                        else if (Hardware.HardwareType == HardwareType.Battery)
                        {
                            Hardware.Update();

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
                        else if (Hardware.HardwareType == HardwareType.Motherboard)
                        {
                            Hardware.Update();

                            SBMI.MotherboardData.State = true;
                            SBMI.MotherboardData.Name = Hardware.Name;
                        }
                    }
                });

                //_ = Task.Run(() =>
                //{
                //    foreach (IHardware Hardware in SBMI.Computer.Hardware)
                //    {
                //        Console.WriteLine("Hardware: {0}", Hardware.Name);

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