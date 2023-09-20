using Newtonsoft.Json.Linq;
using SBMI = Sucrose.Backgroundog.Manage.Internal;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Data
    {
        public static JObject GetCpuInfo()
        {
            return new JObject
            {
                { "Min", SBMI.CpuData.Min },
                { "Now", SBMI.CpuData.Now },
                { "Max", SBMI.CpuData.Max },
                { "Name", SBMI.CpuData.Name },
                { "Core", SBMI.CpuData.Core },
                { "State", SBMI.CpuData.State },
                { "Thread", SBMI.CpuData.Thread },
                { "Fullname", SBMI.CpuData.Fullname }
            };
        }

        public static JObject GetBiosInfo()
        {
            return new JObject
            {
                { "Name", SBMI.BiosData.Name },
                { "State", SBMI.BiosData.State },
                { "Caption", SBMI.BiosData.Caption },
                { "Version", SBMI.BiosData.Version },
                { "Description", SBMI.BiosData.Description },
                { "ReleaseDate", SBMI.BiosData.ReleaseDate },
                { "Manufacturer", SBMI.BiosData.Manufacturer },
                { "SerialNumber", SBMI.BiosData.SerialNumber },
                { "CurrentLanguage", SBMI.BiosData.CurrentLanguage }
            };
        }

        public static JObject GetDateInfo()
        {
            return new JObject
            {
                { "Day", SBMI.DateData.Day },
                { "Hour", SBMI.DateData.Hour },
                { "Year", SBMI.DateData.Year },
                { "State", SBMI.DateData.State },
                { "Month", SBMI.DateData.Month },
                { "Minute", SBMI.DateData.Minute },
                { "Second", SBMI.DateData.Second },
                { "Millisecond", SBMI.DateData.Millisecond }
            };
        }

        public static JObject GetAudioInfo()
        {
            return new JObject
            {
                { "PID", SBMI.AudioData.PID },
                { "State", SBMI.AudioData.State },
                { "Title", SBMI.AudioData.Title },
                { "Artist", SBMI.AudioData.Artist },
                { "Hwnd", $"{SBMI.AudioData.Hwnd}" },
                { "Subtitle", SBMI.AudioData.Subtitle },
                { "AlbumTitle", SBMI.AudioData.AlbumTitle },
                { "TrackNumber", SBMI.AudioData.TrackNumber },
                { "AlbumArtist", SBMI.AudioData.AlbumArtist },
                { "SourceAppId", SBMI.AudioData.SourceAppId },
                { "MediaType", $"{SBMI.AudioData.MediaType}" },
                { "PlaybackRate", SBMI.AudioData.PlaybackRate },
                { "PropsValid", $"{SBMI.AudioData.PropsValid}" },
                { "RepeatMode", $"{SBMI.AudioData.RepeatMode}" },
                { "SourceDeviceId", SBMI.AudioData.SourceDeviceId },
                { "RenderDeviceId", SBMI.AudioData.RenderDeviceId },
                { "ShuffleEnabled", SBMI.AudioData.ShuffleEnabled },
                { "PlaybackCaps", $"{SBMI.AudioData.PlaybackCaps}" },
                { "PlaybackMode", $"{SBMI.AudioData.PlaybackMode}" },
                { "ThumbnailString", SBMI.AudioData.ThumbnailString },
                { "AlbumTrackCount", SBMI.AudioData.AlbumTrackCount },
                { "PlaybackState", $"{SBMI.AudioData.PlaybackState}" },
                { "ThumbnailAddress", SBMI.AudioData.ThumbnailAddress },
                { "EndTime", SBMI.AudioData.EndTime.TotalMilliseconds },
                { "Position", SBMI.AudioData.Position.TotalMilliseconds },
                { "StartTime", SBMI.AudioData.StartTime.TotalMilliseconds },
                { "LastPlayingFileTime", SBMI.AudioData.LastPlayingFileTime },
                { "PositionSetFileTime", SBMI.AudioData.PositionSetFileTime },
                { "MinSeekTime", SBMI.AudioData.MinSeekTime.TotalMilliseconds },
                { "MaxSeekTime", SBMI.AudioData.MaxSeekTime.TotalMilliseconds }
            };
        }

        public static JObject GetMemoryInfo()
        {
            return new JObject
            {
                { "Name", SBMI.MemoryData.Name },
                { "State", SBMI.MemoryData.State },
                { "MemoryUsed", SBMI.MemoryData.MemoryUsed },
                { "MemoryLoad", SBMI.MemoryData.MemoryLoad },
                { "MemoryAvailable", SBMI.MemoryData.MemoryAvailable },
                { "VirtualMemoryUsed", SBMI.MemoryData.VirtualMemoryUsed },
                { "VirtualMemoryLoad", SBMI.MemoryData.VirtualMemoryLoad },
                { "VirtualMemoryAvailable", SBMI.MemoryData.VirtualMemoryAvailable }
            };
        }

        public static JObject GetBatteryInfo()
        {
            return new JObject
            {
                { "Name", SBMI.BatteryData.Name },
                { "State", SBMI.BatteryData.State },
                { "Voltage", SBMI.BatteryData.Voltage },
                { "ChargeRate", SBMI.BatteryData.ChargeRate },
                { "ChargeLevel", SBMI.BatteryData.ChargeLevel },
                { "ChargeCurrent", SBMI.BatteryData.ChargeCurrent },
                { "DischargeRate", SBMI.BatteryData.DischargeRate },
                { "DischargeLevel", SBMI.BatteryData.DischargeLevel },
                { "DischargeCurrent", SBMI.BatteryData.DischargeCurrent },
                { "DegradationLevel", SBMI.BatteryData.DegradationLevel },
                { "DesignedCapacity", SBMI.BatteryData.DesignedCapacity },
                { "RemainingCapacity", SBMI.BatteryData.RemainingCapacity },
                { "FullChargedCapacity", SBMI.BatteryData.FullChargedCapacity },
                { "ChargeDischargeRate", SBMI.BatteryData.ChargeDischargeRate },
                { "ChargeDischargeCurrent", SBMI.BatteryData.ChargeDischargeCurrent },
                { "RemainingTimeEstimated", SBMI.BatteryData.RemainingTimeEstimated }
            };
        }

        public static JObject GetNetworkInfo()
        {
            return new JObject
            {
                { "Name", SBMI.NetworkData.Name },
                { "State", SBMI.NetworkData.State },
                { "Upload", SBMI.NetworkData.Upload },
                { "Download", SBMI.NetworkData.Download },
                { "FormatUploadData", SBMI.NetworkData.FormatUploadData },
                { "FormatDownloadData", SBMI.NetworkData.FormatDownloadData },
                {
                    "UploadData", new JObject
                    {
                        { "Text", SBMI.NetworkData.UploadData.Text },
                        { "Value", SBMI.NetworkData.UploadData.Value },
                        { "Type", $"{SBMI.NetworkData.UploadData.Type}" },
                        { "Short", $"{SBMI.NetworkData.UploadData.Short}" }
                    }
                },
                {
                    "DownloadData", new JObject
                    {
                        { "Text", SBMI.NetworkData.DownloadData.Text },
                        { "Value", SBMI.NetworkData.DownloadData.Value },
                        { "Type", $"{SBMI.NetworkData.DownloadData.Type}" },
                        { "Short", $"{SBMI.NetworkData.DownloadData.Short}" }
                    }
                },
            };
        }

        public static JObject GetMotherboardInfo()
        {
            return new JObject
            {
                { "Name", SBMI.MotherboardData.Name },
                { "State", SBMI.MotherboardData.State },
                { "Product", SBMI.MotherboardData.Product },
                { "Version", SBMI.MotherboardData.Version },
                { "Manufacturer", SBMI.MotherboardData.Manufacturer }
            };
        }
    }
}