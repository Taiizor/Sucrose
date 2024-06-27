namespace Sucrose.Shared.Space.Model
{
    internal class FeedbackData(int Stars, string Message, string Version, string Category)
    {
        public int Stars { get; set; } = Stars;

        public string Message { get; set; } = Message;

        public string Version { get; set; } = Version;

        public string Category { get; set; } = Category;
    }
}