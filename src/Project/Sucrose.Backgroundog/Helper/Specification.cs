using LibreHardwareMonitor.Hardware;
using Newtonsoft.Json;
using Skylark.Enum;
using Skylark.Helper;
using Skylark.Standard.Extension.Storage;
using Sucrose.Backgroundog.Extension;
using System.Management;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
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
                Console.WriteLine("Specification");
                //Libredeki cpu, ram, net bilgileri değişkene atılacak

                SBMI.Computer.Accept(new UpdateVisitor());

                foreach (IHardware Hardware in SBMI.Computer.Hardware)
                {
                    if (Hardware.HardwareType == HardwareType.Cpu)
                    {
                        Hardware.Update();

                        foreach (ISensor Sensor in Hardware.Sensors)
                        {
                            if (Sensor.SensorType == SensorType.Load && Sensor.Name == "CPU Total")
                            {
                                SBMI.CpuData = new()
                                {
                                    Min = Sensor.Min,
                                    Max = Sensor.Max,
                                    Now = Sensor.Value,
                                    Name = Hardware.Name,
                                    Core = SBMI.CpuData.Core,
                                    Thread = SBMI.CpuData.Thread,
                                    Fullname = SBMI.CpuData.Fullname
                                };
                            }
                        }

                        if (SBMI.CpuManagement)
                        {
                            SBMI.CpuManagement = false;

                            _ = Task.Run(() =>
                            {
                                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

                                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                                {
                                    SBMI.CpuData.Core = Convert.ToInt32(Object["NumberOfCores"]);
                                    SBMI.CpuData.Fullname = Object["Name"].ToString().TrimStart().TrimEnd();
                                    SBMI.CpuData.Thread = Convert.ToInt32(Object["NumberOfLogicalProcessors"]);
                                    break;
                                }
                            });
                        }
                    }
                    else if (Hardware.HardwareType == HardwareType.Memory)
                    {
                        Hardware.Update();

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

                        SBMI.MotherboardData = new()
                        {
                            Name = Hardware.Name
                        };
                    }
                }

                _ = Task.Run(() =>
                {
                    foreach (string Name in SSSHN.InstanceNetworkInterfaces())
                    {
                        if (SMMM.NetworkAdapter == Name)
                        {
                            if (SMMM.NetworkAdapter != SBMI.NetworkData.Name)
                            {
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
                });

                //Console.WriteLine(JsonConvert.SerializeObject(Data.GetCpuInfo(), Formatting.Indented));
                //Console.WriteLine(JsonConvert.SerializeObject(Data.GetDateInfo(), Formatting.Indented));
                //Console.WriteLine(JsonConvert.SerializeObject(Data.GetMemoryInfo(), Formatting.Indented));
                //Console.WriteLine(JsonConvert.SerializeObject(Data.GetBatteryInfo(), Formatting.Indented));
                Console.WriteLine(JsonConvert.SerializeObject(Data.GetNetworkInfo(), Formatting.Indented));
                //Console.WriteLine(JsonConvert.SerializeObject(Data.GetMotherboardInfo(), Formatting.Indented));

                //foreach (IHardware Hardware in SBMI.Computer.Hardware)
                //{
                //    Console.WriteLine("Hardware: {0}", Hardware.Name);

                //    foreach (IHardware Subhardware in Hardware.SubHardware)
                //    {
                //        Console.WriteLine("\tSubhardware: {0}", Subhardware.Name);

                //        foreach (ISensor Sensor in Subhardware.Sensors)
                //        {
                //            Console.WriteLine("\t\tSensor: {0}, type: {1}, value: {2}", Sensor.Name, Sensor.SensorType, Sensor.Value);
                //        }
                //    }

                //    foreach (ISensor Sensor in Hardware.Sensors)
                //    {
                //        Console.WriteLine("\tSensor: {0}, type: {1}, value: {2}", Sensor.Name, Sensor.SensorType, Sensor.Value);
                //    }
                //}

                Console.WriteLine("----------------------------------------------");
            }

            await Task.CompletedTask;
        }

        private class UpdateVisitor : IVisitor
        {
            public void VisitComputer(IComputer Computer)
            {
                Computer.Traverse(this);
            }

            public void VisitHardware(IHardware Hardware)
            {
                Hardware.Update();

                foreach (IHardware SubHardware in Hardware.SubHardware)
                {
                    SubHardware.Accept(this);
                }
            }

            public void VisitSensor(ISensor Sensor) { }

            public void VisitParameter(IParameter Parameter) { }
        }
    }
}