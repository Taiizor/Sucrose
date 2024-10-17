using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class FeedbackOptionalData
    {
        [JsonProperty("VotedStar", Required = Required.Always)]
        public int VotedStar { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }

        [JsonProperty("ContactEmail", Required = Required.Always)]
        public string ContactEmail { get; set; }

        [JsonProperty("WrittenMessage", Required = Required.Always)]
        public string WrittenMessage { get; set; }

        [JsonProperty("RelatedCategory", Required = Required.Always)]
        public string RelatedCategory { get; set; }
    }
}