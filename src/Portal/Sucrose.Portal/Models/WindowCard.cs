using Wpf.Ui.Controls;

namespace Sucrose.Portal.Models
{
    public record WindowCard
    {
        public string Name { get; set; }

        public string Value { get; set; }

        public string Description { get; set; }

        public SymbolRegular Icon { get; set; }

        public WindowCard(string name, string value, string description, SymbolRegular icon)
        {
            Icon = icon;
            Name = name;
            Value = value;
            Description = description;
        }
    }
}