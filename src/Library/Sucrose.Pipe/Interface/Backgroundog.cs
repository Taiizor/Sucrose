using Newtonsoft.Json.Linq;

namespace Sucrose.Pipe.Interface
{
    public class Backgroundog
    {
        public JObject Cpu { get; set; } = null;

        public JObject Bios { get; set; } = null;

        public JObject Date { get; set; } = null;

        public JObject Audio { get; set; } = null;

        public JObject Memory { get; set; } = null;

        public JObject Battery { get; set; } = null;

        public JObject Graphic { get; set; } = null;

        public JObject Network { get; set; } = null;

        public JObject Motherboard { get; set; } = null;
    }
}