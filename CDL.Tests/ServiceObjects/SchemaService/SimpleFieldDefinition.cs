namespace CDL.Tests.ServiceObjects.SchemaService
{
    public class SimpleFiled
    {
        public SimpleItem simple { get; set; }
    }

    public class SimpleItem
    {
        public string field_name { get; set; } 
        public string field_type { get; set; } 
    }
}