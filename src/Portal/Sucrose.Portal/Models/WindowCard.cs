using Wpf.Ui.Controls;

namespace Sucrose.Portal.Models
{
    public record WindowCard
    {
        public string Name { get; set; }

        public string Description { get; set; }

        public SymbolRegular Icon { get; set; }

        public string Value { get; set; }

        public WindowCard(string name, string description, SymbolRegular icon, string value)
        {
            Name = name;
            Description = description;
            Icon = icon;
            Value = value;
        }
    }
}