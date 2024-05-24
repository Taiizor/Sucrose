namespace Sucrose.Shared.Store.Interface
{
    internal class Data(double progressPercentage, int downloadedFileCount, int totalFileCount, string percentage, string state, string guid)
    {
        public double ProgressPercentage { get; set; } = progressPercentage;

        public int DownloadedFileCount { get; set; } = downloadedFileCount;

        public int TotalFileCount { get; set; } = totalFileCount;

        public string Percentage { get; set; } = percentage;

        public string State { get; set; } = state;

        public string Guid { get; set; } = guid;
    }
}