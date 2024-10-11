namespace Sucrose.Shared.Space.Model
{
    internal class FeedbackData(int VotedStar, string UserMessage, string AppVersion, string RelatedCategory)
    {
        public int VotedStar { get; set; } = VotedStar;

        public string AppVersion { get; set; } = AppVersion;

        public string UserMessage { get; set; } = UserMessage;

        public string RelatedCategory { get; set; } = RelatedCategory;
    }
}